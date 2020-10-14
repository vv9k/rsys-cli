use super::cmd::{RsysCmd, RsysOpt};
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
        if let Some(cmd) = self.opts.cmd.clone() {
            match cmd {
                RsysCmd::Get(opts) => self.get(opts)?,
                RsysCmd::Dump(opts) => self.dump(opts)?,
                RsysCmd::Watch(opts) => self.watch(opts)?,
                RsysCmd::Show { cmd } => self.show(cmd.clone()),
            }
        }

        Ok(())
    }
}
