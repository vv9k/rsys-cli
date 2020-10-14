use super::DumpOpts;
use crate::{
    cmd::common::SystemInfo,
    util::{print, PrintFormat},
    RsysCli,
};
use rsys::Result;

impl RsysCli {
    pub fn dump(&self, opts: DumpOpts) -> Result<()> {
        let format = if opts.json {
            PrintFormat::Json
        } else if opts.yaml {
            PrintFormat::Yaml
        } else {
            PrintFormat::Normal
        };
        print(
            SystemInfo::new(
                &self.system,
                true,
                true,
                true,
                true,
                true,
                true,
                opts.cpu,
                opts.memory,
                opts.network,
                opts.storage,
                opts.mounts,
                opts.all,
                opts.stats,
                opts.processes,
            )?,
            format,
            opts.pretty,
        )
    }
}
