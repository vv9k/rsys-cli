use super::{
    opt::{RsysCmd, RsysOpt},
    util::PrintFormat,
};
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
                RsysCmd::Get { property, json, pretty } if *json => self.get(property, PrintFormat::Json, *pretty)?,
                RsysCmd::Get { property, json, pretty } if !json => self.get(property, PrintFormat::Normal, *pretty)?,
                RsysCmd::Dump {
                    json,
                    pretty,
                    memory,
                    network,
                    storage,
                    mounts,
                } if *json => self.dump(PrintFormat::Json, *pretty, *memory, *network, *storage, *mounts)?,
                RsysCmd::Dump {
                    json,
                    pretty,
                    memory,
                    network,
                    storage,
                    mounts,
                } if !json => self.dump(PrintFormat::Normal, *pretty, *memory, *network, *storage, *mounts)?,
                _ => {}
            }
        }

        Ok(())
    }
}
