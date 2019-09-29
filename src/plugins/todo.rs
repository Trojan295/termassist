use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use clap::{App, Arg, ArgMatches, SubCommand};
use linked_hash_map::LinkedHashMap;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::widgets::{Block, SelectableList, Widget};
use tui::Terminal;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::plugins::Plugin;
use crate::ui::{Event, Events};

const PLUGIN_NAME: &str = "todo";

pub struct TodoPlugin {
    filename: String,
}

struct TodoApp {
    pub todos: Vec<String>,
    pub position: usize,
    pub to_delete: Option<usize>,
    pub should_quit: bool,
}

struct TodoUI {}

impl TodoApp {
    fn new(items: &Vec<String>) -> TodoApp {
        let mut options = items.clone();
        options.push("Exit".to_owned());
        TodoApp {
            todos: options,
            position: 0,
            to_delete: None,
            should_quit: false,
        }
    }

    fn on_down(&mut self) {
        if self.todos.len() - 1 != self.position {
            self.position += 1;
        }
    }

    fn on_up(&mut self) {
        if self.position != 0 {
            self.position -= 1;
        }
    }

    fn on_cancel(&mut self) {
        self.should_quit = true;
    }

    fn on_return(&mut self) {
        self.should_quit = true;
        self.to_delete = if self.position == self.todos.len() - 1 {
            None
        } else {
            Some(self.position)
        }
    }
}

impl TodoUI {
    pub fn render_done<B>(terminal: &mut Terminal<B>, app: &TodoApp) -> std::io::Result<()>
    where
        B: tui::backend::Backend,
    {
        terminal.draw(|mut f| {
            let size = f.size();
            SelectableList::default()
                .block(Block::default().title("Mark done:"))
                .items(&app.todos)
                .select(Some(app.position))
                .style(Style::default())
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(&mut f, size);
        })
    }
}

fn create_terminal() -> std::io::Result<
    Terminal<tui::backend::TermionBackend<termion::raw::RawTerminal<std::io::Stdout>>>,
> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

impl Plugin for TodoPlugin {
    fn name(&self) -> String {
        PLUGIN_NAME.to_string()
    }

    fn register_cli<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.subcommand(
            SubCommand::with_name(PLUGIN_NAME)
                .subcommand(
                    SubCommand::with_name("add")
                        .arg(Arg::with_name("message").required(true).multiple(true)),
                )
                .subcommand(SubCommand::with_name("done")),
        )
    }

    fn command<'a>(&self, matches: &ArgMatches<'a>) {
        match matches.subcommand() {
            ("add", Some(args)) => {
                let messages = args.values_of("message").unwrap();
                let message: String =
                    messages.fold("".to_owned(), |acc, x| format!("{} {}", acc, x));
                self.add(&message.trim()).expect("Cannot add item");
            }
            ("done", Some(_)) => {
                let mut terminal = create_terminal().unwrap();
                let mut app = TodoApp::new(&self.list().unwrap());
                let events = Events::default();

                loop {
                    TodoUI::render_done(&mut terminal, &app).unwrap();

                    match events.next() {
                        Ok(Event::Input(key)) => match key {
                            Key::Down => {
                                app.on_down();
                            }
                            Key::Up => {
                                app.on_up();
                            }
                            Key::Char('\n') => {
                                app.on_return();
                            }
                            Key::Ctrl('c') => {
                                app.on_cancel();
                            }
                            _ => {}
                        },
                        Ok(Event::Tick) => {}
                        _ => {}
                    }
                    if app.should_quit {
                        break;
                    }
                }
                match app.to_delete {
                    None => {}
                    Some(pos) => {
                        self.remove(pos).unwrap();
                    }
                };
            }
            (_, _) => {}
        };
    }

    fn show(&self) -> Option<String> {
        self.list()
            .map(|messages| match messages.len() {
                0 => None,
                _ => {
                    let mut res = String::from("----- TODO ------");
                    for msg in messages.iter() {
                        res.push_str(&format!("\n- {}", msg));
                    }
                    Some(res)
                }
            })
            .unwrap_or_else(|err| Some(format!("Error in TodoPlugin show(): {}", err)))
    }
}

impl TodoPlugin {
    pub fn new() -> TodoPlugin {
        let home = std::env::var("HOME").expect("Cannot find $HOME env variable");
        TodoPlugin {
            filename: format!("{}/.local/share/termassist/todo.yml", home),
        }
    }

    fn open_file(&self) -> std::io::Result<File> {
        let path = Path::new(&self.filename);
        std::fs::create_dir_all(path.parent().unwrap())?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.filename)
    }

    fn add(&self, message: &str) -> std::io::Result<u32> {
        let file = self.open_file()?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents).unwrap();

        let messages = match docs.len() {
            0 => vec![Yaml::from_str(message)],
            _ => {
                let x = &docs[0];
                let current = x["messages"].clone();
                current
                    .into_vec()
                    .map(|mut msgs| {
                        msgs.push(Yaml::String(message.to_string()));
                        msgs
                    })
                    .unwrap()
            }
        };

        let item_id = messages.len() as u32;
        let mut letters = LinkedHashMap::new();
        letters.insert(Yaml::String("messages".to_string()), Yaml::Array(messages));

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&Yaml::Hash(letters)).unwrap();

        let mut file = File::create(&self.filename)?;
        file.write_all(out_str.as_bytes())?;

        Ok(item_id)
    }

    fn remove(&self, id: usize) -> std::io::Result<()> {
        let file = self.open_file()?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents).unwrap();

        let doc = &docs[0];

        let messages = doc["messages"].clone();

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        let new_msgs = messages
            .into_vec()
            .map(|mut msgs| {
                msgs.remove(id);
                msgs
            })
            .unwrap();

        let mut letters = LinkedHashMap::new();
        letters.insert(Yaml::String("messages".to_string()), Yaml::Array(new_msgs));

        emitter.dump(&Yaml::Hash(letters)).unwrap();

        let mut file = File::create(&self.filename)?;
        file.write_all(out_str.as_bytes())?;

        Ok(())
    }

    fn list(&self) -> std::io::Result<Vec<String>> {
        let file = self.open_file()?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let mut messages = vec![];

        let docs = YamlLoader::load_from_str(&contents).unwrap();
        if docs.len() == 0 {
            return Ok(messages);
        }

        let doc = &docs[0];
        doc["messages"].as_vec().map(|yaml_messages| {
            for m in yaml_messages {
                messages.push(m.as_str().unwrap().to_string())
            }
        });

        Ok(messages)
    }
}
