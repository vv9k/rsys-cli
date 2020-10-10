use crate::{
    cli::RsysCli,
    util::{print, PrintFormat},
};
use rsys::linux::{
    net::Interface,
    ps::{processes, Process},
    storage::{BlockStorageDeviceName, DeviceMapper, MultipleDeviceStorage, ScsiCdrom, StorageDevice},
};
use rsys::Result;
use structopt::StructOpt;

#[allow(non_camel_case_types)]
#[derive(StructOpt)]
pub enum Property {
    /// Cpu architecture
    arch,
    /// All cpu stats and cores
    cpu,
    cpu_model,
    cpu_clock,
    cpu_cores,
    domain,
    hostname,
    /// Lookup statistics and information about network interface
    interface {
        /// Name of the interface to lookup. For example `eth0` or `enp8s0`
        name: String,
    },
    interfaces,
    kernel,
    logical_cores,
    /// All memory statistics
    memory,
    memory_free,
    memory_total,
    /// Mountpoints from /etc/mounts
    mounts,
    os,
    pid {
        id: i32,
    },
    /// Prints the first process that contains name in its cmdline
    process {
        /// Process name
        name: String,
    },
    /// Storage device info
    storage {
        /// Name of the storage device. For example `sda` or `md0`
        name: String,
    },
    swap_free,
    swap_total,
    uptime,
}

impl RsysCli {
    pub fn get(&self, property: &Property, format: PrintFormat, pretty: bool) -> Result<()> {
        use Property::*;
        match property {
            arch => print(self.system.arch()?, format, pretty)?,
            cpu => print(self.system.processor()?, format, pretty)?,
            cpu_model => print(self.system.cpu()?, format, pretty)?,
            cpu_clock => print(self.system.cpu_clock()?, format, pretty)?,
            cpu_cores => print(self.system.cpu_cores()?, format, pretty)?,
            domain => print(self.system.domainname()?, format, pretty)?,
            hostname => print(self.system.hostname()?, format, pretty)?,
            interface { name } => {
                if let Some(iface) = self.get_interface(name) {
                    print(iface, format, pretty)?;
                } else {
                    println!("Interface `{}` not found", name);
                }
            }
            interfaces => print(self.system.ifaces()?, format, pretty)?,
            kernel => print(self.system.kernel_version()?, format, pretty)?,
            logical_cores => print(self.system.logical_cores()?, format, pretty)?,
            os => print(self.system.os(), format, pretty)?,
            memory => print(self.system.memory()?, format, pretty)?,
            memory_free => print(self.system.memory_free()?, format, pretty)?,
            memory_total => print(self.system.memory_total()?, format, pretty)?,
            mounts => print(self.system.mounts()?, format, pretty)?,
            pid { id } => print(Process::new(*id)?, format, pretty)?,
            process { name } => {
                for ps in processes()? {
                    if ps.cmdline.contains(name) {
                        print(ps, format, pretty)?;
                        break;
                    }
                }
            }
            storage { name } => self.print_storage(name, format, pretty)?,
            swap_total => print(self.system.swap_total()?, format, pretty)?,
            swap_free => print(self.system.swap_free()?, format, pretty)?,
            uptime => print(self.system.uptime()?, format, pretty)?,
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

    fn get_interface(&self, name: &str) -> Option<Interface> {
        if let Some(interface) = self.system.ifaces().ok()?.0.iter().filter(|i| i.name == name).next() {
            return Some(interface.clone());
        }
        None
    }
}
