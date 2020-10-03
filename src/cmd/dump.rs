use crate::{
    cli::RsysCli,
    util::{handle_err, print, PrintFormat},
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
    kernel: String,
    cpu: Option<Processor>,
    memory: Option<Memory>,
    mounts: Option<MountPoints>,
    interaces: Option<Interfaces>,
    storage_devices: Option<StorageDevices>,
    multiple_device_storages: Option<MultipleDeviceStorages>,
    device_mappers: Option<DeviceMappers>,
}
impl SystemInfo {
    fn new(
        r: &Rsys,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        mounts: bool,
        all: bool,
        stats: bool,
    ) -> Result<SystemInfo> {
        Ok(Self {
            arch: handle_err(r.arch()),
            hostname: handle_err(r.hostname()),
            domain: handle_err(r.domainname()),
            uptime: handle_err(r.uptime()),
            os: r.os(),
            kernel: handle_err(r.kernel_version()),
            cpu: if cpu || all {
                Some(handle_err(r.processor()))
            } else {
                None
            },
            memory: if memory || all {
                Some(handle_err(r.memory()))
            } else {
                None
            },
            mounts: if mounts || all {
                Some(handle_err(r.mounts()))
            } else {
                None
            },
            interaces: if net || all { Some(handle_err(r.ifaces())) } else { None },
            storage_devices: if storage || all {
                Some(handle_err(storage_devices::<StorageDevice>(stats)))
            } else {
                None
            },
            multiple_device_storages: if storage || all {
                Some(handle_err(storage_devices::<MultipleDeviceStorage>(stats)))
            } else {
                None
            },
            device_mappers: if storage || all {
                Some(handle_err(storage_devices::<DeviceMapper>(stats)))
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
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        mounts: bool,
        all: bool,
        stats: bool,
    ) -> Result<()> {
        print(
            SystemInfo::new(&self.system, cpu, memory, net, storage, mounts, all, stats)?,
            format,
            pretty,
        )
    }
}
