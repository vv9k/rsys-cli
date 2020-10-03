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
                RsysCmd::Get {
                    property,
                    json,
                    yaml: _,
                    pretty,
                } if *json => self.get(property, PrintFormat::Json, *pretty)?,
                RsysCmd::Get {
                    property,
                    json: _,
                    yaml,
                    pretty,
                } if *yaml => self.get(property, PrintFormat::Yaml, *pretty)?,
                RsysCmd::Get {
                    property,
                    json,
                    yaml,
                    pretty,
                } if !yaml && !json => self.get(property, PrintFormat::Normal, *pretty)?,
                RsysCmd::Dump {
                    json,
                    yaml: _,
                    pretty,
                    cpu,
                    memory,
                    network,
                    storage,
                    mounts,
                    all,
                } if *json => self.dump(
                    PrintFormat::Json,
                    *pretty,
                    *cpu,
                    *memory,
                    *network,
                    *storage,
                    *mounts,
                    *all,
                )?,
                RsysCmd::Dump {
                    json: _,
                    yaml,
                    pretty,
                    cpu,
                    memory,
                    network,
                    storage,
                    mounts,
                    all,
                } if *yaml => self.dump(
                    PrintFormat::Yaml,
                    *pretty,
                    *cpu,
                    *memory,
                    *network,
                    *storage,
                    *mounts,
                    *all,
                )?,
                RsysCmd::Dump {
                    json,
                    yaml,
                    pretty,
                    cpu,
                    memory,
                    network,
                    storage,
                    mounts,
                    all,
                } if !yaml && !json => self.dump(
                    PrintFormat::Normal,
                    *pretty,
                    *cpu,
                    *memory,
                    *network,
                    *storage,
                    *mounts,
                    *all,
                )?,
                _ => {}
            }
        }

        Ok(())
    }
}
