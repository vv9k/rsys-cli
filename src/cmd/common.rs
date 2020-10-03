use crate::util::handle_err;
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
pub(crate) struct SystemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uptime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<Processor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<Memory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mounts: Option<MountPoints>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interfaces: Option<Interfaces>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_devices: Option<StorageDevices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiple_device_storages: Option<MultipleDeviceStorages>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_mappers: Option<DeviceMappers>,
}
impl SystemInfo {
    pub(crate) fn new(
        r: &Rsys,
        arch: bool,
        hostname: bool,
        domain: bool,
        uptime: bool,
        os: bool,
        kernel: bool,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        mounts: bool,
        all: bool,
        stats: bool,
    ) -> Result<SystemInfo> {
        Ok(Self {
            arch: if arch || all { Some(handle_err(r.arch())) } else { None },
            hostname: if hostname || all {
                Some(handle_err(r.hostname()))
            } else {
                None
            },
            domain: if domain || all {
                Some(handle_err(r.domainname()))
            } else {
                None
            },
            uptime: if uptime || all {
                Some(handle_err(r.uptime()))
            } else {
                None
            },
            os: if os || all { Some(r.os()) } else { None },
            kernel: if kernel || all {
                Some(handle_err(r.kernel_version()))
            } else {
                None
            },
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
            interfaces: if net || all { Some(handle_err(r.ifaces())) } else { None },
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
