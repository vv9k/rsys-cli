use crate::{
    cmd::common::SystemInfo,
    util::{print, PrintFormat},
    RsysCli,
};
use rsys::Result;

impl RsysCli {
    pub(crate) fn dump(
        &self,
        format: PrintFormat,
        pretty: bool,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        mounts: bool,
        all: bool,
        stats: bool,
        processes: bool,
    ) -> Result<()> {
        print(
            SystemInfo::new(
                &self.system,
                true,
                true,
                true,
                true,
                true,
                true,
                cpu,
                memory,
                net,
                storage,
                mounts,
                all,
                stats,
                processes,
            )?,
            format,
            pretty,
        )
    }
}
