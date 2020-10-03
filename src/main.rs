#[macro_use]
extern crate prettytable;
pub(crate) mod cli;
pub(crate) mod cmd;
pub(crate) mod util;
use cli::RsysCli;
use rsys::Result;

fn main() -> Result<()> {
    let rsys = RsysCli::new();
    if let Err(e) = rsys.main() {
        eprintln!("Error: {}", e);
    }

    Ok(())
}
