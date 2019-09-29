extern crate chrono;
extern crate clap;
extern crate linked_hash_map;
extern crate tui;
extern crate yaml_rust;

mod plugins;
mod ui;

use clap::{App, SubCommand};

use plugins::{Plugin, ReminderPlugin, TodoPlugin};

fn main() {
    let plugins: Vec<Box<Plugin>> =
        vec![Box::new(TodoPlugin::new()), Box::new(ReminderPlugin::new())];

    let mut app = App::new("termassist")
        .version("0.1.0")
        .author("Damian Czaja <trojan295@gmail.com>")
        .subcommand(SubCommand::with_name("show").about("Renders the termassist message"));

    for plugin in plugins.iter() {
        app = plugin.register_cli(app);
    }

    let matches = app.get_matches();

    match matches.subcommand() {
        ("show", _) => {
            for plugin in plugins {
                match plugin.show() {
                    None => {}
                    Some(msg) => println!("{}", msg),
                }
            }
        }
        (plugin_name, Some(matches)) => {
            for plugin in plugins.iter() {
                if plugin.name() == plugin_name {
                    plugin.command(matches);
                }
            }
        }
        (&_, None) => println!("Wrong params. Use --help"),
    }
}
