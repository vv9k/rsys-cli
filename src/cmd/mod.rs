pub mod common;
pub mod dump;
pub mod get;
pub mod show;
pub mod watch;
use get::Property;
pub use show::ShowCmd;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "rsys", about = "Aquire all important information about your system")]
pub struct RsysOpt {
    #[structopt(subcommand)]
    /// Command to run
    pub cmd: Option<RsysCmd>,
}

#[derive(StructOpt)]
pub enum RsysCmd {
    /// Prints out a system proprty to stdout
    Get {
        #[structopt(subcommand)]
        /// One of system properties
        property: Property,
        #[structopt(short, long)]
        /// Print output as JSON
        json: bool,
        #[structopt(short, long)]
        /// Print output as YAML
        yaml: bool,
        #[structopt(short, long)]
        /// Make the output pretty
        pretty: bool,
    },
    /// Dumps all information
    Dump {
        #[structopt(short, long)]
        /// Print output as JSON
        json: bool,
        #[structopt(short, long)]
        /// Print output as YAML
        yaml: bool,
        #[structopt(short, long)]
        /// Make the output pretty
        pretty: bool,
        #[structopt(long)]
        /// Include CPU info with cores
        cpu: bool,
        #[structopt(long)]
        /// Include memory statistics
        memory: bool,
        #[structopt(long)]
        /// Adds network interfaces to the output
        network: bool,
        #[structopt(long)]
        /// Adds info about storage devices, device mappers,
        /// multiple device arrays
        storage: bool,
        #[structopt(long)]
        /// Adds all processes
        processes: bool,
        #[structopt(long)]
        /// Whether to parse stats for all storage devices or just the main ones.
        /// Only functional with `--storage` and `network` flag
        stats: bool,
        #[structopt(long)]
        /// Adds information about mountpoints on host os
        mounts: bool,
        #[structopt(short, long)]
        /// Shortcut for `--cpu --memory --storage --network --mounts --stats --processes`
        all: bool,
    },
    /// Monitor specified parameters. Default parameters are hostname and uptime.
    /// To monitor more parameters use flags like `cpu`, `memory` or `storage`.
    /// This command runs indefinitely unless a `duration` parameter is specified
    /// and by default prints JSON with parameters each second. To change how often
    /// there is a snapshot of data adjust `interval` parameter.
    Watch {
        #[structopt(short, long)]
        /// Make the output pretty
        pretty: bool,
        #[structopt(long)]
        /// Include CPU info with cores
        cpu: bool,
        #[structopt(long)]
        /// Include memory statistics
        memory: bool,
        #[structopt(long)]
        /// Adds network interfaces to the output
        network: bool,
        #[structopt(long)]
        /// Adds info about storage devices, device mappers,
        /// multiple device arrays
        storage: bool,
        #[structopt(long)]
        /// Whether to parse stats for all storage devices or just the main ones.
        /// Only functional with `--storage` flag
        stats: bool,
        #[structopt(short, long)]
        /// Shortcut for `--cpu --memory --storage --network --mounts`
        all: bool,
        #[structopt(short, long)]
        /// Duration in seconds for which to collect data. Default is 18_446_744_073_709_551_615 seconds
        duration: Option<u64>,
        #[structopt(short, long)]
        /// How long to wait between runs in milliseconds. Default is 1000
        interval: Option<u64>,
    },
    /// Dashboard mode with graphs and interactive lists
    Show {
        #[structopt(subcommand)]
        /// What dashboard to show
        cmd: ShowCmd,
    },
}
