extern crate clap;
extern crate linked_hash_map;
extern crate yaml_rust;

mod plugins;

use clap::{App, SubCommand};

use plugins::{Plugin, TodoPlugin};

fn main() {
    let mut plugins = vec![TodoPlugin::new()];

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
            for mut plugin in plugins {
                match plugin.show() {
                    None => {}
                    Some(msg) => println!("{}", msg),
                }
            }
        }
        (plugin_name, Some(matches)) => {
            for plugin in plugins.iter_mut() {
                if plugin.name() == plugin_name {
                    match plugin.command(matches) {
                        None => {}
                        Some(msg) => println!("{}", msg),
                    }
                }
            }
        }
        (&_, None) => println!("Wrong params. Use --help"),
    }
}
