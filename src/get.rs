use super::cli::RsysCli;
use rsys::linux::{BlockStorageDeviceName, DeviceMapper, MultipleDeviceStorage, ScsiCdrom, StorageDevice};
use rsys::{Error, Result};
use serde::Serialize;
use serde_json as json;
use std::{
    any::type_name,
    fmt::{Debug, Display},
};
use structopt::StructOpt;

fn json_to_string<T: Serialize>(val: T, pretty: bool) -> Result<String> {
    let f = if pretty {
        json::to_string_pretty
    } else {
        json::to_string
    };

    f(&val).map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))
}

fn print<T: Debug + Display + Serialize>(val: T, format: PrintFormat, pretty: bool) -> Result<()> {
    match format {
        PrintFormat::Normal => {
            if pretty {
                print!("{:#?}", val);
            } else {
                print!("{}", val);
            }
        }
        PrintFormat::Json => {
            print!("{}", json_to_string(val, pretty)?);
        }
    }

    Ok(())
}

#[allow(non_camel_case_types)]
#[derive(StructOpt)]
pub enum Property {
    /// Cpu architecture
    arch,
    hostname,
    domain,
    uptime,
    os,
    /// All cpu stats and cores
    cpu,
    cpu_model,
    cpu_clock,
    cpu_cores,
    logical_cores,
    /// All memory statistics
    memory,
    memory_free,
    memory_total,
    /// Mountpoints from /etc/mounts
    mounts,
    process {
        /// Id of the process to stat
        pid: i32,
    },
    /// Storage device info
    storage {
        /// Name of the storage device. For example `sda` or `md0`
        name: String,
    },
    swap_free,
    swap_total,
}

pub(crate) enum PrintFormat {
    Normal,
    Json,
}

impl RsysCli {
    pub(crate) fn get(&self, property: &Property, format: PrintFormat, pretty: bool) -> Result<()> {
        use Property::*;
        match property {
            hostname => print(self.system.hostname()?, format, pretty)?,
            domain => print(self.system.domainname()?, format, pretty)?,
            uptime => print(self.system.uptime()?, format, pretty)?,
            os => print(self.system.os(), format, pretty)?,
            arch => print(self.system.arch()?, format, pretty)?,
            cpu => print(self.system.processor()?, format, pretty)?,
            cpu_model => print(self.system.cpu()?, format, pretty)?,
            cpu_clock => print(self.system.cpu_clock()?, format, pretty)?,
            cpu_cores => print(self.system.cpu_cores()?, format, pretty)?,
            logical_cores => print(self.system.logical_cores()?, format, pretty)?,
            memory => print(self.system.memory()?, format, pretty)?,
            memory_free => print(self.system.memory_free()?, format, pretty)?,
            memory_total => print(self.system.memory_total()?, format, pretty)?,
            mounts => print(self.system.mounts()?, format, pretty)?,
            process { pid } => print(self.system.stat_process(*pid)?, format, pretty)?,
            storage { name } => self.print_storage(name, format, pretty)?,
            swap_total => print(self.system.swap_total()?, format, pretty)?,
            swap_free => print(self.system.swap_free()?, format, pretty)?,
        }
        Ok(())
    }

    fn print_storage(&self, name: &str, format: PrintFormat, pretty: bool) -> Result<()> {
        if name.starts_with(StorageDevice::prefix()) {
            print(self.system.stat_block_device(name)?, format, pretty)?
        } else if name.starts_with(DeviceMapper::prefix()) {
            print(self.system.stat_device_mapper(name)?, format, pretty)?
        } else if name.starts_with(MultipleDeviceStorage::prefix()) {
            print(self.system.stat_multiple_device_storage(name)?, format, pretty)?
        } else if name.starts_with(ScsiCdrom::prefix()) {
            print(self.system.stat_scsi_cdrom(name)?, format, pretty)?
        }

        Ok(())
    }
}
