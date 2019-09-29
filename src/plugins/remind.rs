use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use chrono::prelude::*;
use clap::{App, Arg, ArgMatches, SubCommand};
use linked_hash_map::LinkedHashMap;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::plugins::Plugin;

const PLUGIN_NAME: &str = "remind";

pub struct ReminderPlugin {
    filename: String,
}

struct ReminderNote {
    date: Date<Local>,
    message: String,
}

impl ReminderNote {
    pub fn to_yaml(&self) -> Yaml {
        let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();

        let date = format!("{} 00:00:00", self.date.format("%Y-%m-%d"));
        map.insert(Yaml::String("date".to_owned()), Yaml::String(date));

        map.insert(
            Yaml::String("message".to_owned()),
            Yaml::String(self.message.to_string()),
        );

        Yaml::Hash(map)
    }

    pub fn from_yaml(yaml: &Yaml) -> ReminderNote {
        let date = yaml["date"].as_str().unwrap();
        ReminderNote {
            date: Local
                .datetime_from_str(date, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .date(),
            message: yaml["message"].as_str().unwrap().to_owned(),
        }
    }
}

impl Plugin for ReminderPlugin {
    fn name(&self) -> String {
        String::from(PLUGIN_NAME)
    }

    fn register_cli<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.subcommand(
            SubCommand::with_name(PLUGIN_NAME).subcommand(
                SubCommand::with_name("add")
                    .arg(Arg::with_name("date").required(true))
                    .arg(Arg::with_name("message").required(true)),
            ),
        )
    }

    fn show(&self) -> Option<String> {
        self.open_file()
            .map(|docs| self.read_notes(docs))
            .map(|yaml_notes| yaml_notes.iter().map(ReminderNote::from_yaml).collect())
            .ok()
            .and_then(|notes: Vec<ReminderNote>| match notes.len() {
                0 => None,
                _ => {
                    let now = Local::now();
                    let mut res = String::from("----- REMINDERS ------");
                    for note in notes.iter() {
                        if now.date() == note.date {
                            res.push_str(&format!("\n  {}", note.message));
                        }
                    }
                    Some(res)
                }
            })
    }

    fn command<'a>(&self, matches: &ArgMatches<'a>) {
        match matches.subcommand() {
            ("add", Some(args)) => {
                let date = args
                    .value_of("date")
                    .map(|date_str| {
                        Local.datetime_from_str(
                            &format!("{} 00:00:00", date_str),
                            "%Y-%m-%d %H:%M:%S",
                        )
                    })
                    .unwrap()
                    .expect("Wrong date format");
                let message = matches.value_of("message").unwrap();

                let note = ReminderNote {
                    date: date.date(),
                    message: String::from(message),
                };
                self.add(&note);
            }
            (&_, _) => {}
        }
    }
}

impl ReminderPlugin {
    pub fn new() -> ReminderPlugin {
        let home = std::env::var("HOME").expect("Cannot find $HOME env variable");
        ReminderPlugin {
            filename: format!("{}/.local/share/termassist/remind.yml", home),
        }
    }

    fn open_file(&self) -> Result<Vec<Yaml>, String> {
        let path = Path::new(&self.filename);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.filename)
            .map_err(|e| e.to_string())?;

        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;

        YamlLoader::load_from_str(&contents).map_err(|e| e.to_string())
    }

    fn save_file(&self, doc: &Yaml) -> Result<(), String> {
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).map_err(|e| e.to_string())?;

        let mut file = File::create(&self.filename).map_err(|e| e.to_string())?;
        file.write_all(out_str.as_bytes())
            .map_err(|e| e.to_string())
    }

    fn read_notes(&self, docs: Vec<Yaml>) -> Vec<Yaml> {
        match docs.len() {
            0 => vec![],
            _ => docs[0]["reminders"].clone().into_vec().unwrap_or(vec![]),
        }
    }

    fn write_notes(&self, notes: Vec<Yaml>) -> Yaml {
        let mut doc = LinkedHashMap::new();
        doc.insert(Yaml::String("reminders".to_string()), Yaml::Array(notes));
        Yaml::Hash(doc)
    }

    fn add(&self, note: &ReminderNote) -> Result<(), String> {
        let mut notes = self.open_file().map(|docs| self.read_notes(docs))?;
        notes.push(note.to_yaml());
        self.save_file(&self.write_notes(notes))
    }
}
