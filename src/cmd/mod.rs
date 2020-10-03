pub(crate) mod dump;
pub(crate) mod get;
use get::Property;
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
        /// Whether to parse stats for all storage devices or just the main ones.
        /// Only functional with `--storage` flag
        stats: bool,
        #[structopt(long)]
        /// Adds information about mountpoints on host os
        mounts: bool,
        #[structopt(short, long)]
        /// Shortcut for `--cpu --memory --storage --network --mounts`
        all: bool,
    },
}
