#[macro_use]
extern crate prettytable;
pub mod cli;
pub mod cmd;
pub mod util;
use cli::RsysCli;

fn main() {
    let rsys = RsysCli::new();
    if let Err(e) = rsys.main() {
        eprintln!("Error: {}", e);
    }
}
