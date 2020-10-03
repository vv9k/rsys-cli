use crate::{
    cli::RsysCli,
    util::{handle_err, print, PrintFormat},
};
use rsys::{
    linux::{
        cpu::Processor,
        mem::Memory,
        net::Interfaces,
        storage::{
            storage_devices, DeviceMapper, DeviceMappers, MultipleDeviceStorage, MultipleDeviceStorages, StorageDevice,
            StorageDevices,
        },
    },
    Result, Rsys,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Formatter},
    thread,
    time::{Duration, Instant},
    u64,
};

#[derive(Debug, Serialize, Deserialize)]
struct MonitorStats {
    hostname: String,
    uptime: u64,
    cpu: Option<Processor>,
    memory: Option<Memory>,
    interaces: Option<Interfaces>,
    storage_devices: Option<StorageDevices>,
    multiple_device_storages: Option<MultipleDeviceStorages>,
    device_mappers: Option<DeviceMappers>,
}
impl MonitorStats {
    fn new(
        r: &Rsys,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        all: bool,
        stats: bool,
    ) -> Result<MonitorStats> {
        Ok(Self {
            hostname: handle_err(r.hostname()),
            uptime: handle_err(r.uptime()),
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
impl fmt::Display for MonitorStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RsysCli {
    pub(crate) fn watch(
        &self,
        pretty: bool,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        all: bool,
        stats: bool,
        duration: Option<u64>,
        interval: Option<u64>,
    ) -> Result<()> {
        let duration = if let Some(d) = duration {
            Duration::from_secs(d)
        } else {
            Duration::from_secs(u64::MAX)
        };
        let interval = if let Some(i) = interval { i } else { 1000 };
        let loop_start = Instant::now();
        loop {
            let print_start = Instant::now();
            print(
                MonitorStats::new(&self.system, cpu, memory, net, storage, all, stats)?,
                PrintFormat::Json,
                pretty,
            )?;
            print!("\n");
            let print_duration = print_start.elapsed().as_millis();
            if loop_start.elapsed() > duration {
                break;
            }
            let sleep_duration = Duration::from_millis((interval as u128 - print_duration) as u64);
            thread::sleep(sleep_duration);
        }
        Ok(())
    }
}
