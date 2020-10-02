use super::cli::RsysCli;
use rsys::linux::{BlockStorageDeviceName, DeviceMapper, MultipleDeviceStorage, ScsiCdrom, StorageDevice};
use rsys::{Error, Result};
use serde::Serialize;
use serde_json as json;
use std::any::type_name;
use structopt::StructOpt;

fn json_to_string<T: Serialize>(val: T, pretty: bool) -> Result<String> {
    let f = if pretty {
        json::to_string_pretty
    } else {
        json::to_string
    };

    f(&val).map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))
}

#[allow(non_camel_case_types)]
#[derive(StructOpt)]
pub enum Property {
    hostname,
    domain,
    uptime,
    os,
    arch,
    cpu,
    cpu_clock,
    cpu_cores,
    logical_cores,
    memory,
    memory_free,
    memory_total,
    mounts,
    process { pid: i32 },
    storage { name: String },
    swap_free,
    swap_total,
}

impl RsysCli {
    pub(crate) fn get(&self, property: &Property) -> Result<()> {
        use Property::*;
        match property {
            hostname => print!("{}", self.system.hostname()?),
            domain => print!("{}", self.system.domainname()?),
            uptime => print!("{}", self.system.uptime()?),
            os => print!("{}", self.system.os()),
            arch => print!("{}", self.system.arch()?),
            cpu => print!("{}", self.system.cpu()?),
            cpu_clock => print!("{}", self.system.cpu_clock()?),
            cpu_cores => print!("{}", self.system.cpu_cores()?),
            logical_cores => print!("{}", self.system.logical_cores()?),
            memory => print!("{:#?}", self.system.memory()?),
            memory_free => print!("{}", self.system.memory_free()?),
            memory_total => print!("{}", self.system.memory_total()?),
            mounts => print!("{:#?}", self.system.mounts()?),
            process { pid } => print!("{:#?}", self.system.stat_process(*pid)?),
            storage { name } => self.print_storage(name)?,
            swap_total => print!("{}", self.system.swap_total()?),
            swap_free => print!("{}", self.system.swap_free()?),
        }
        Ok(())
    }

    pub(crate) fn get_json(&self, property: &Property, pretty: bool) -> Result<()> {
        use Property::*;
        match property {
            hostname => print!("{}", json_to_string(self.system.hostname()?, pretty)?),
            domain => print!("{}", json_to_string(self.system.domainname()?, pretty)?),
            uptime => print!("{}", json_to_string(self.system.uptime()?, pretty)?),
            os => print!("{}", json_to_string(self.system.os(), pretty)?),
            arch => print!("{}", json_to_string(self.system.arch()?, pretty)?),
            cpu => print!("{}", json_to_string(self.system.cpu()?, pretty)?),
            cpu_clock => print!("{}", json_to_string(self.system.cpu_clock()?, pretty)?),
            cpu_cores => print!("{}", json_to_string(self.system.cpu_cores()?, pretty)?),
            logical_cores => print!("{}", json_to_string(self.system.logical_cores()?, pretty)?),
            memory => print!("{}", json_to_string(self.system.memory()?, pretty)?),
            memory_free => print!("{}", json_to_string(self.system.memory_free()?, pretty)?),
            memory_total => print!("{}", json_to_string(self.system.memory_total()?, pretty)?),
            mounts => print!("{}", json_to_string(self.system.mounts()?, pretty)?),
            process { pid } => print!("{}", json_to_string(self.system.stat_process(*pid)?, pretty)?),
            storage { name } => self.print_json_storage(name, pretty)?,
            swap_free => print!("{}", json_to_string(self.system.swap_free()?, pretty)?),
            swap_total => print!("{}", json_to_string(self.system.swap_total()?, pretty)?),
        }

        Ok(())
    }

    fn print_storage(&self, name: &str) -> Result<()> {
        if name.starts_with(StorageDevice::prefix()) {
            print!("{:#?}", self.system.stat_block_device(name)?);
        } else if name.starts_with(DeviceMapper::prefix()) {
            print!("{:#?}", self.system.stat_device_mapper(name)?);
        } else if name.starts_with(MultipleDeviceStorage::prefix()) {
            print!("{:#?}", self.system.stat_multiple_device_storage(name)?);
        } else if name.starts_with(ScsiCdrom::prefix()) {
            print!("{:#?}", self.system.stat_scsi_cdrom(name)?);
        }

        Ok(())
    }

    fn print_json_storage(&self, name: &str, pretty: bool) -> Result<()> {
        if name.starts_with(StorageDevice::prefix()) {
            print!("{}", json_to_string(self.system.stat_block_device(name)?, pretty)?);
        } else if name.starts_with(DeviceMapper::prefix()) {
            print!("{}", json_to_string(self.system.stat_device_mapper(name)?, pretty)?);
        } else if name.starts_with(MultipleDeviceStorage::prefix()) {
            print!(
                "{}",
                json_to_string(self.system.stat_multiple_device_storage(name)?, pretty)?
            );
        } else if name.starts_with(ScsiCdrom::prefix()) {
            print!("{}", json_to_string(self.system.stat_scsi_cdrom(name)?, pretty)?);
        }

        Ok(())
    }
}
