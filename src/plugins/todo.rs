use clap::{App, Arg, ArgMatches, SubCommand};
use linked_hash_map::LinkedHashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::plugins::Plugin;

const TODO_PLUGIN_NAME: &str = "todo";

pub struct TodoPlugin {
    filename: String,
}

impl Plugin for TodoPlugin {
    fn name(&self) -> String {
        TODO_PLUGIN_NAME.to_string()
    }

    fn register_cli<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.subcommand(
            SubCommand::with_name(TODO_PLUGIN_NAME)
                .subcommand(
                    SubCommand::with_name("add").arg(Arg::with_name("message").required(true)),
                )
                .subcommand(SubCommand::with_name("done").arg(Arg::with_name("id").required(true))),
        )
    }

    fn command<'a>(&mut self, matches: &ArgMatches<'a>) -> String {
        match matches.subcommand() {
            ("add", Some(args)) => {
                let message = args.value_of("message").unwrap();
                self.add(message).expect("Cannot add item");
            }
            ("done", Some(args)) => {
                let item_id = args
                    .value_of("id")
                    .unwrap()
                    .parse::<u32>()
                    .expect("ID must be a number");
                self.remove(item_id).expect("Cannot remove item");
            }
            (_, _) => {}
        };
        self.show()
    }

    fn show(&mut self) -> String {
        let result = self.list().map(|messages| {
            let mut res = String::from("----- TODO ------\n");
            for (i, msg) in messages.iter().enumerate() {
                res.push_str(&format!("  {}. {}\n", i + 1, msg));
            }
            res
        });
        match result {
            Ok(msg) => msg,
            Err(err) => format!("Error in TodoPlugin show(): {}", err),
        }
    }
}

impl TodoPlugin {
    pub fn new() -> TodoPlugin {
        let home = std::env::var("HOME").expect("Cannot find $HOME env variable");
        let filepath = format!("{}/.local/share/termassist/todo.yml", home);
        TodoPlugin { filename: filepath }
    }

    fn open_file(&mut self) -> std::io::Result<File> {
        let path = Path::new(&self.filename);
        std::fs::create_dir_all(path.parent().unwrap())?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.filename)
    }

    fn add(&mut self, message: &str) -> std::io::Result<u32> {
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

    fn remove(&mut self, id: u32) -> std::io::Result<()> {
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
                msgs.remove((id - 1) as usize);
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

    fn list(&mut self) -> std::io::Result<Vec<String>> {
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
