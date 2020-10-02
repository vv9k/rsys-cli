use super::opt::{RsysCmd, RsysOpt};
use rsys::{Result, Rsys};
use structopt::StructOpt;

pub struct RsysCli {
    pub opts: RsysOpt,
    pub system: Rsys,
}
impl RsysCli {
    pub fn new() -> RsysCli {
        RsysCli {
            opts: RsysOpt::from_args(),
            system: Rsys::new(),
        }
    }

    pub fn main(&self) -> Result<()> {
        if let Some(cmd) = &self.opts.cmd {
            match cmd {
                RsysCmd::Get {
                    property,
                    json,
                    pretty: _,
                } if !(*json) => self.get(property)?,
                RsysCmd::Get { property, json, pretty } if *json => self.get_json(property, *pretty)?,
                _ => {}
            }
        }

        Ok(())
    }
}
