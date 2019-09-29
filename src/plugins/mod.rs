mod remind;
mod todo;

use clap::{App, ArgMatches};
pub use remind::ReminderPlugin;
pub use todo::TodoPlugin;

pub trait Plugin {
    fn name(&self) -> String;
    fn register_cli<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn show(&self) -> Option<String>;
    fn command<'a>(&self, matches: &ArgMatches<'a>);
}
