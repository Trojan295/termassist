use clap::{App, ArgMatches};

mod todo;
pub use todo::TodoPlugin;

pub trait Plugin {
    fn show(&mut self) -> String;
    fn name(&self) -> String;
    fn register_cli<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn command<'a>(&mut self, matches: &ArgMatches<'a>) -> String;
}
