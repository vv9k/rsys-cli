use super::GetOpts;
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
#[derive(StructOpt, Clone)]
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
    ps {
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
    pub fn get(&self, opts: GetOpts) -> Result<()> {
        use Property::*;
        let format = PrintFormat::from_bools(opts.json, opts.yaml);
        match opts.property {
            arch => print(self.system.arch()?, format, opts.pretty)?,
            cpu => print(self.system.processor()?, format, opts.pretty)?,
            cpu_model => print(self.system.cpu()?, format, opts.pretty)?,
            cpu_clock => print(self.system.cpu_clock()?, format, opts.pretty)?,
            cpu_cores => print(self.system.cpu_cores()?, format, opts.pretty)?,
            domain => print(self.system.domainname()?, format, opts.pretty)?,
            hostname => print(self.system.hostname()?, format, opts.pretty)?,
            interface { name } => {
                if let Some(iface) = self.get_interface(&name) {
                    print(iface, format, opts.pretty)?;
                } else {
                    println!("Interface `{}` not found", name);
                }
            }
            interfaces => print(self.system.ifaces()?, format, opts.pretty)?,
            kernel => print(self.system.kernel_version()?, format, opts.pretty)?,
            logical_cores => print(self.system.logical_cores()?, format, opts.pretty)?,
            os => print(self.system.os(), format, opts.pretty)?,
            memory => print(self.system.memory()?, format, opts.pretty)?,
            memory_free => print(self.system.memory_free()?, format, opts.pretty)?,
            memory_total => print(self.system.memory_total()?, format, opts.pretty)?,
            mounts => print(self.system.mounts()?, format, opts.pretty)?,
            pid { id } => print(Process::new(id)?, format, opts.pretty)?,
            ps { name } => {
                for process in processes()? {
                    if process.cmdline.contains(&name) {
                        print(process, format, opts.pretty)?;
                        break;
                    }
                }
            }
            storage { name } => self.print_storage(&name, format, opts.pretty)?,
            swap_total => print(self.system.swap_total()?, format, opts.pretty)?,
            swap_free => print(self.system.swap_free()?, format, opts.pretty)?,
            uptime => print(self.system.uptime()?, format, opts.pretty)?,
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
