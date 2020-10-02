use json::json;
use rsys::{
    linux::{
        BlockStorageDeviceName, DeviceMapper, Memory, MountPoints, MultipleDeviceStorage, Process, ScsiCdrom,
        StorageDevice,
    },
    Error, Result, Rsys,
};
use serde::Serialize;
use serde_json as json;
use std::any::type_name;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "rsys",
    about = "Aquire all important information about your operating system"
)]
pub struct RsysOpt {
    #[structopt(subcommand)]
    pub cmd: Option<RsysCmd>,
}

#[derive(StructOpt)]
pub enum RsysCmd {
    Get {
        #[structopt(subcommand)]
        property: Property,
        #[structopt(short)]
        json: bool,
    },
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
    mounts,
    process { pid: i32 },
    storage { name: String },
    swap_total,
    swap_free,
}

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
                RsysCmd::Get { property, json } if !(*json) => self.get(property)?,
                RsysCmd::Get { property, json } if *json => self.get_json(property)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn get(&self, property: &Property) -> Result<()> {
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
            mounts => print!("{:#?}", self.system.mounts()?),
            process { pid } => print!("{:#?}", self.system.stat_process(*pid)?),
            storage { name } => self.print_storage(name)?,
            swap_total => print!("{}", self.system.swap_total()?),
            swap_free => print!("{}", self.system.swap_free()?),
        }

        Ok(())
    }

    fn get_json(&self, property: &Property) -> Result<()> {
        use Property::*;
        match property {
            hostname => print!("{}", json!({"hostname": self.system.hostname()?})),
            domain => print!("{}", json!({"domain": self.system.domainname()?})),
            uptime => print!("{}", json!({"uptime": self.system.uptime()?})),
            os => print!("{}", json!({"os": self.system.os()})),
            arch => print!("{}", json!({"arch": self.system.arch()?})),
            cpu => print!("{}", json!({"cpu": self.system.cpu()?})),
            cpu_clock => print!("{}", json!({"cpu_clock": self.system.cpu_clock()?})),
            cpu_cores => print!("{}", json!({"cpu_cores": self.system.cpu_cores()?})),
            logical_cores => print!("{}", json!({"logical_cores": self.system.logical_cores()?})),
            memory => print!("{}", json_to_string_pretty(self.system.memory()?)?),
            mounts => print!("{}", json_to_string_pretty(self.system.mounts()?)?),
            process { pid } => print!("{:#?}", json_to_string_pretty(self.system.stat_process(*pid)?)),
            storage { name } => self.print_json_storage(name)?,
            swap_total => print!("{}", json!({"swap_total": self.system.swap_total()?})),
            swap_free => print!("{}", json!({"swap_free": self.system.swap_free()?})),
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

    fn print_json_storage(&self, name: &str) -> Result<()> {
        if name.starts_with(StorageDevice::prefix()) {
            print!("{}", json_to_string_pretty(self.system.stat_block_device(name)?)?);
        } else if name.starts_with(DeviceMapper::prefix()) {
            print!("{}", json_to_string_pretty(self.system.stat_device_mapper(name)?)?);
        } else if name.starts_with(MultipleDeviceStorage::prefix()) {
            print!(
                "{}",
                json_to_string_pretty(self.system.stat_multiple_device_storage(name)?)?
            );
        } else if name.starts_with(ScsiCdrom::prefix()) {
            print!("{}", json_to_string_pretty(self.system.stat_scsi_cdrom(name)?)?);
        }

        Ok(())
    }
}

fn json_to_string_pretty<T: Serialize>(val: T) -> Result<String> {
    json::to_string_pretty(&val).map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))
}

fn main() -> Result<()> {
    let rsys = RsysCli::new();
    if let Err(e) = rsys.main() {
        eprintln!("Error: {}", e);
    }

    Ok(())
}
