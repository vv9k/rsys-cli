# rsys-cli
[![Build Status](https://travis-ci.com/wojciechkepka/rsys-cli.svg?branch=master)](https://travis-ci.com/wojciechkepka/rsys-cli)  
CLI tool for quick access to system information. For now Linux only.

## Available commands
### `get`
Gets a specified parameter.
```
USAGE:
    rsys get [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -j, --json       Print output as JSON
    -p, --pretty     Make the output pretty
    -V, --version    Prints version information
    -y, --yaml       Print output as YAML

SUBCOMMANDS:
    arch             Cpu architecture
    cpu              All cpu stats and cores
    cpu-clock
    cpu-cores
    cpu-model
    domain
    help             Prints this message or the help of the given subcommand(s)
    hostname
    interface        Lookup statistics and information about network interface
    interfaces
    kernel
    logical-cores
    memory           All memory statistics
    memory-free
    memory-total
    mounts           Mountpoints from /etc/mounts
    os
    process
    storage          Storage device info
    swap-free
    swap-total
    uptime
```
### Example usage and output
`rsys get -jp memory`  
```
{
  "total": 16712667136,
  "free": 6789361664,
  "available": 12793421824,
  "buffers": 263999488,
  "cached": 5953527808,
  "active": 5261893632,
  "inactive": 3771269120,
  "shared": 232402944
}
```
`rsys get -p interface enp8s0`  
```
Interface {
    name: "enp8s0",
    ipv4: "192.168.0.1",
    stat: IfaceStat {
        rx_bytes: 1263128140,
        rx_packets: 929371,
        rx_errs: 0,
        rx_drop: 0,
        rx_fifo: 0,
        rx_frame: 0,
        rx_compressed: 0,
        rx_multicast: 15519,
        tx_bytes: 47660514,
        tx_packets: 555310,
        tx_errs: 0,
        tx_drop: 0,
        tx_fifo: 0,
        tx_frame: 0,
        tx_compressed: 0,
        tx_multicast: 0,
    },
    mtu: 1500,
    mac_address: "70:85:c2:f9:9b:2a",
    speed: 1000,
}
```
### `watch`
```
Monitor specified parameters. Default parameters are hostname and uptime. To monitor more parameters use flags like
`cpu`, `memory` or `storage`. This command runs indefinitely unless a `duration` parameter is specified and by default
prints JSON with parameters each second. To change how often there is a snapshot of data adjust `interval` parameter

USAGE:
    rsys watch [FLAGS] [OPTIONS]

FLAGS:
    -a, --all        Shortcut for `--cpu --memory --storage --network --mounts`
        --cpu        Include CPU info with cores
    -h, --help       Prints help information
        --memory     Include memory statistics
        --network    Adds network interfaces to the output
    -p, --pretty     Make the output pretty
        --stats      Whether to parse stats for all storage devices or just the main ones. Only functional with
                     `--storage` flag
        --storage    Adds info about storage devices, device mappers, multiple device arrays
    -V, --version    Prints version information

OPTIONS:
    -d, --duration <duration>    Duration in seconds for which to collect data. Default is 18_446_744_073_709_551_615
                                 seconds
    -i, --interval <interval>    How long to wait between runs in milliseconds. Default is 1000
```
### `dump`                                                                 
Dumps all data in specified format. By default only basic info like
hostname, uptime, cpu architecture are dumped. To enable more information
use `--memory`, `--mounts`, `--storage`, `--network` flags
```
USAGE:
    rsys dump [FLAGS]

FLAGS:
    -a, --all        Shortcut for `--cpu --memory --storage --network --mounts`
        --cpu        Include CPU info with cores
    -h, --help       Prints help information
    -j, --json       Print output as JSON
        --memory     Include memory statistics
        --mounts     Adds information about mountpoints on host os
        --network    Adds network interfaces to the output
    -p, --pretty     Make the output pretty
        --stats      Whether to parse stats for all storage devices or just the main ones. Only functional with
                     `--storage` flag
        --storage    Adds info about storage devices, device mappers, multiple device arrays
    -V, --version    Prints version information
    -y, --yaml       Print output as YAML
```

### Some examples
#### Basic dump in YAML
```
$ rsys dump -y
---
arch: x86_64
hostname: arch
domain: (none)
uptime: 4861
os: linux
kernel: 5.8.12-arch1-1
```

#### Dumping all information as pretty json (Output from this benchmark [here](https://github.com/wojciechkepka/rsys-cli/blob/master/example_output/dump)
```
$ hyperfine --warmup 5 'rsys dump -jp --all'
Benchmark #1: rsys dump -jp --all
  Time (mean ± σ):       2.5 ms ±   0.2 ms    [User: 1.6 ms, System: 1.5 ms]
  Range (min … max):     1.0 ms …   3.4 ms    553 runs
```



## License
[**MIT**](https://github.com/wojciechkepka/rsys-cli/blob/master/LICENSE)
