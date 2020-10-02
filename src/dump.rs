use super::{
    cli::RsysCli,
    util::{print, PrintFormat},
};
use rsys::{
    linux::{Memory, MountPoints, Processor},
    Result, Rsys,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    arch: String,
    hostname: String,
    domain: String,
    uptime: u64,
    os: String,
    cpu: Processor,
    kernel: String,
    memory: Memory,
    mounts: MountPoints,
}
impl SystemInfo {
    fn new(r: &Rsys) -> Result<SystemInfo> {
        Ok(Self {
            arch: r.arch()?,
            hostname: r.hostname()?,
            domain: r.domainname()?,
            uptime: r.uptime()?,
            os: r.os(),
            cpu: r.processor()?,
            kernel: r.kernel_version()?,
            memory: r.memory()?,
            mounts: r.mounts()?,
        })
    }
}
impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RsysCli {
    pub(crate) fn dump(&self, format: PrintFormat, pretty: bool) -> Result<()> {
        print(SystemInfo::new(&self.system)?, format, pretty)
    }
}
