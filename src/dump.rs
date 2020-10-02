use super::{
    cli::RsysCli,
    util::{print, PrintFormat},
};
use rsys::{
    linux::{
        cpu::Processor,
        mem::Memory,
        misc::MountPoints,
        net::Interfaces,
        storage::{
            storage_devices, DeviceMapper, DeviceMappers, MultipleDeviceStorage, MultipleDeviceStorages, StorageDevice,
            StorageDevices,
        },
    },
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
    memory: Option<Memory>,
    mounts: Option<MountPoints>,
    interaces: Option<Interfaces>,
    storage_devices: Option<StorageDevices>,
    multiple_device_storages: Option<MultipleDeviceStorages>,
    device_mappers: Option<DeviceMappers>,
}
impl SystemInfo {
    fn new(r: &Rsys, memory: bool, net: bool, storage: bool, mounts: bool) -> Result<SystemInfo> {
        Ok(Self {
            arch: r.arch()?,
            hostname: r.hostname()?,
            domain: r.domainname()?,
            uptime: r.uptime()?,
            os: r.os(),
            cpu: r.processor()?,
            kernel: r.kernel_version()?,
            memory: if memory { Some(r.memory()?) } else { None },
            mounts: if mounts { Some(r.mounts()?) } else { None },
            interaces: if net { Some(r.ifaces()?) } else { None },
            storage_devices: if storage {
                Some(storage_devices::<StorageDevice>()?)
            } else {
                None
            },
            multiple_device_storages: if storage {
                Some(storage_devices::<MultipleDeviceStorage>()?)
            } else {
                None
            },
            device_mappers: if storage {
                Some(storage_devices::<DeviceMapper>()?)
            } else {
                None
            },
        })
    }
}
impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RsysCli {
    pub(crate) fn dump(
        &self,
        format: PrintFormat,
        pretty: bool,
        memory: bool,
        net: bool,
        storage: bool,
        mounts: bool,
    ) -> Result<()> {
        print(
            SystemInfo::new(&self.system, memory, net, storage, mounts)?,
            format,
            pretty,
        )
    }
}
