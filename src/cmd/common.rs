use crate::util::{conv_b, conv_hz, handle_err};
use prettytable::{format, Table};
use rsys::{
    linux::{
        cpu::Processor,
        mem::Memory,
        misc::MountPoints,
        net::Interfaces,
        ps::Processes,
        storage::{
            storage_devices, DeviceMapper, DeviceMappers, MultipleDeviceStorage, MultipleDeviceStorages, StorageDevice,
            StorageDevices,
        },
    },
    Result, Rsys,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

const SECTOR_SIZE: u64 = 512;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
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
    processes: Option<Processes>,
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
    #[serde(skip_serializing)]
    display_stats: bool,
    #[serde(skip_serializing)]
    display_all: bool,
}
impl SystemInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        processes: bool,
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
            processes: if processes || all {
                Some(handle_err(r.processes()))
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
                let show_stats = if all { true } else { stats };
                Some(handle_err(storage_devices::<StorageDevice>(show_stats)))
            } else {
                None
            },
            multiple_device_storages: if storage || all {
                let show_stats = if all { true } else { stats };
                Some(handle_err(storage_devices::<MultipleDeviceStorage>(show_stats)))
            } else {
                None
            },
            device_mappers: if storage || all {
                let show_stats = if all { true } else { stats };
                Some(handle_err(storage_devices::<DeviceMapper>(show_stats)))
            } else {
                None
            },
            display_stats: stats,
            display_all: all,
        })
    }
    fn general_section_string(&self) -> String {
        let mut s = String::new();
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        if let Some(hostname) = &self.hostname {
            table.add_row(row!["hostname:", l -> hostname]);
        }
        if let Some(arch) = &self.arch {
            table.add_row(row!["arch:", l -> arch]);
        }
        if let Some(domain) = &self.domain {
            if domain != "(none)" {
                table.add_row(row!["domain:", l -> domain]);
            }
        }
        if let Some(kernel) = &self.kernel {
            table.add_row(row!["kernel:", l -> kernel]);
        }
        if let Some(uptime) = &self.uptime {
            table.add_row(row!["uptime:", l -> format!("{} s", uptime)]);
        }
        if let Some(os) = &self.os {
            table.add_row(row!["os:", l -> os]);
        }
        s.push_str(" GENERAL:\n");
        s.push_str(&table.to_string());
        s
    }
    fn cpu_section_string(&self) -> String {
        let mut s = String::new();
        if let Some(cpu) = &self.cpu {
            s.push_str(" CPU:\n");
            let mut cpu_table = Table::new();
            let mut cores_table = Table::new();
            cpu_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            cores_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            cpu_table.add_row(row!["model:", cpu.model]);
            cpu_table.add_row(row!["cache size:", conv_b(cpu.cache_size)]);
            cpu_table.add_row(row!["bogomips:", cpu.bogomips]);
            cores_table.add_row(row![c => "core", "minimum", "current", "max"]);

            for core in &cpu.cores {
                cores_table.add_row(row![
                    &format!("cpu{}", core.id),
                    conv_hz(core.min_freq),
                    conv_hz(core.cur_freq),
                    conv_hz(core.max_freq),
                ]);
            }
            s.push_str(&cpu_table.to_string());
            s.push_str(&cores_table.to_string());
        }
        s
    }
    fn memory_section_string(&self) -> String {
        let mut s = String::new();
        if let Some(memory) = &self.memory {
            s.push_str(" MEMORY:\n");
            let mut mem_table = Table::new();
            mem_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            mem_table.add_row(row!["total:", r -> conv_b(memory.total)]);
            mem_table.add_row(row!["free:", r -> conv_b(memory.free)]);
            mem_table.add_row(row!["available:", r -> conv_b(memory.available)]);
            mem_table.add_row(row!["buffers:", r -> conv_b(memory.buffers)]);
            mem_table.add_row(row!["cached:", r -> conv_b(memory.cached)]);
            mem_table.add_row(row!["active:", r -> conv_b(memory.active)]);
            mem_table.add_row(row!["inactive:", r -> conv_b(memory.inactive)]);
            mem_table.add_row(row!["shared:", r -> conv_b(memory.shared)]);
            s.push_str(&mem_table.to_string());
        }
        s
    }
    fn network_section_string(&self) -> String {
        let mut s = String::new();
        if let Some(ifaces) = &self.interfaces {
            s.push_str(" NETWORK:\n");
            let mut net_table = Table::new();
            let mut stats_table = Table::new();
            net_table.set_format(*format::consts::FORMAT_NO_LINESEP);
            stats_table.set_format(*format::consts::FORMAT_NO_LINESEP);

            net_table.add_row(row![ c => "name", "ipv4", "ipv6", "mac", "speed", "mtu",]);
            stats_table.add_row(row![ c =>
                "name",
                "bytes",
                "packets",
                "errs",
                "drop",
                "fifo",
                "frame",
                "compressed",
                "multicast",
            ]);
            for iface in &ifaces.0 {
                net_table.add_row(row![
                    iface.name,
                    iface.ipv4,
                    iface.ipv6,
                    iface.mac_address,
                    format!("{} mb/s", iface.speed),
                    iface.mtu,
                ]);
                stats_table.add_row(row![
                    iface.name,
                    c -> format!("{} / {}", conv_b(iface.stat.rx_bytes), conv_b(iface.stat.tx_bytes)),
                    c -> format!("{} / {}", iface.stat.rx_packets, iface.stat.tx_packets),
                    c -> format!("{} / {}", iface.stat.rx_errs, iface.stat.tx_errs),
                    c -> format!("{} / {}", iface.stat.rx_drop, iface.stat.tx_drop),
                    c -> format!("{} / {}", iface.stat.rx_fifo, iface.stat.tx_fifo),
                    c -> format!("{} / {}", iface.stat.rx_frame, iface.stat.tx_frame),
                    c -> format!("{} / {}", iface.stat.rx_compressed, iface.stat.tx_compressed),
                    c -> format!("{} / {}", iface.stat.rx_multicast, iface.stat.tx_multicast),
                ]);
            }
            s.push_str(&net_table.to_string());
            if self.display_stats || self.display_all {
                s.push_str(" NETWORK STATS: ( rx / tx - received / transfered )\n");
                s.push_str(&stats_table.to_string());
            }
        }
        s
    }
    fn storage_section_string(&self) -> String {
        let mut s = String::new();
        if let Some(storages) = &self.storage_devices {
            s.push_str(" STORAGE DEVICES:\n");
            let mut storage_table = Table::new();
            let mut stats_table = Table::new();
            storage_table.set_format(*format::consts::FORMAT_NO_LINESEP);
            stats_table.set_format(*format::consts::FORMAT_NO_LINESEP);

            storage_table.add_row(row![
                c =>
                "name",
                "size",
                "major",
                "min",
                "block size",
                "model",
                "vendor",
                "state"
            ]);
            stats_table.add_row(row![
                c =>
                "device",
                "r I/O's",
                "r merges",
                "r sectors",
                "r ticks",
                "w I/O's",
                "w merges",
                "w sectors",
                "w ticks",
                "d I/O's",
                "d merges",
                "d sectors",
                "d ticks",
                "in flight",
                "I/O ticks",
            ]);
            for storage in storages {
                storage_table.add_row(row![
                    storage.info.dev,
                    r -> conv_b(storage.info.size as u64 * SECTOR_SIZE),
                    storage.info.maj,
                    storage.info.min,
                    storage.info.block_size,
                    storage.model,
                    storage.vendor,
                    storage.state
                ]);

                if let Some(stat) = &storage.info.stat {
                    stats_table.add_row(row![
                        storage.info.dev,
                        stat.read_ios,
                        stat.read_merges,
                        stat.read_sectors,
                        stat.read_ticks,
                        stat.write_ios,
                        stat.write_merges,
                        stat.write_sectors,
                        stat.write_ticks,
                        stat.discard_ios,
                        stat.discard_merges,
                        stat.discard_sectors,
                        stat.discard_ticks,
                        stat.in_flight,
                        stat.io_ticks
                    ]);
                }
            }
            s.push_str(&storage_table.to_string());
            if let Some(mds) = &self.multiple_device_storages {
                s.push_str(" MULTIPLE DEVICE ARRAYS:\n");
                let mut mds_table = Table::new();
                mds_table.set_format(*format::consts::FORMAT_NO_LINESEP);

                mds_table.add_row(row![ c => "name", "size", "major", "min", "block size", "level",]);
                for md in mds {
                    mds_table.add_row(row![
                        md.info.dev,
                        r -> conv_b(md.info.size as u64 * SECTOR_SIZE),
                        md.info.maj,
                        md.info.min,
                        md.info.block_size,
                        md.level,
                    ]);
                    if let Some(stat) = &md.info.stat {
                        stats_table.add_row(row![
                            md.info.dev,
                            stat.read_ios,
                            stat.read_merges,
                            stat.read_sectors,
                            stat.read_ticks,
                            stat.write_ios,
                            stat.write_merges,
                            stat.write_sectors,
                            stat.write_ticks,
                            stat.discard_ios,
                            stat.discard_merges,
                            stat.discard_sectors,
                            stat.discard_ticks,
                            stat.in_flight,
                            stat.io_ticks
                        ]);
                    }
                }
                s.push_str(&mds_table.to_string());
            }
            if let Some(dms) = &self.device_mappers {
                s.push_str(" DEVICE MAPPERS:\n");
                let mut dms_table = Table::new();
                dms_table.set_format(*format::consts::FORMAT_NO_LINESEP);

                dms_table.add_row(row![ c => "name", "size", "major", "min", "block size", "vname", "uuid",]);
                for dm in dms {
                    dms_table.add_row(row![
                        dm.info.dev,
                        r -> conv_b(dm.info.size as u64 * SECTOR_SIZE),
                        dm.info.maj,
                        dm.info.min,
                        dm.info.block_size,
                        dm.name,
                        dm.uuid,
                    ]);
                    if let Some(stat) = &dm.info.stat {
                        stats_table.add_row(row![
                            dm.info.dev,
                            stat.read_ios,
                            stat.read_merges,
                            stat.read_sectors,
                            stat.read_ticks,
                            stat.write_ios,
                            stat.write_merges,
                            stat.write_sectors,
                            stat.write_ticks,
                            stat.discard_ios,
                            stat.discard_merges,
                            stat.discard_sectors,
                            stat.discard_ticks,
                            stat.in_flight,
                            stat.io_ticks
                        ]);
                    }
                }
                s.push_str(&dms_table.to_string());
            }
            if self.display_stats || self.display_all {
                s.push_str(" STORAGE STATS: (r - read, w - write, d - discard)\n");
                s.push_str(&stats_table.to_string());
            }
        }
        s
    }
    fn processes_section_string(&self) -> String {
        let mut s = String::new();
        if let Some(processes) = &self.processes {
            s.push_str(" PROCESSES:\n");
            let mut p_table = Table::new();
            p_table.set_format(*format::consts::FORMAT_NO_LINESEP);
            p_table.add_row(row![
                c =>
                "pid",
                "cmdline",
                "name",
                "state",
                "ppid",
                "pgrp",
                "session",
                "tty_nr",
                "utime",
                "stime",
                "cutime",
                "cstime",
                "priority",
                "nice",
                "num_threads",
                "itrealvalue",
                "starttime",
                "vsize",
                "rss",
                "rsslim",
                "nswap",
                "cnswap",
                "guest_time",
                "cguest_time",
                "processor"
            ]);

            for p in processes {
                p_table.add_row(row![
                    p.stat.pid,
                    p.cmdline,
                    p.stat.name,
                    p.stat.state,
                    p.stat.ppid,
                    p.stat.pgrp,
                    p.stat.session,
                    p.stat.tty_nr,
                    p.stat.utime,
                    p.stat.stime,
                    p.stat.cutime,
                    p.stat.cstime,
                    p.stat.priority,
                    p.stat.nice,
                    p.stat.num_threads,
                    p.stat.itrealvalue,
                    p.stat.starttime,
                    p.stat.vsize,
                    p.stat.rss,
                    p.stat.rsslim,
                    p.stat.nswap,
                    p.stat.cnswap,
                    p.stat.guest_time,
                    p.stat.cguest_time,
                    p.stat.processor
                ]);
            }
            s.push_str(&p_table.to_string());
        }

        s
    }
}
impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&self.general_section_string());
        s.push_str(&self.cpu_section_string());
        s.push_str(&self.memory_section_string());
        s.push_str(&self.network_section_string());
        s.push_str(&self.storage_section_string());
        s.push_str(&self.processes_section_string());
        write!(f, "{}", s)
    }
}
