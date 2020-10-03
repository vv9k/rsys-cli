use super::get::Property;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "rsys",
    about = "Aquire all important information about your operating system"
)]
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
        /// Make the output pretty
        pretty: bool,
    },
    /// Dumps all information
    Dump {
        #[structopt(short, long)]
        /// Print output as JSON
        json: bool,
        #[structopt(short, long)]
        /// Make the output pretty
        pretty: bool,
        #[structopt(long)]
        cpu: bool,
        #[structopt(long)]
        memory: bool,
        #[structopt(long)]
        network: bool,
        #[structopt(long)]
        storage: bool,
        #[structopt(long)]
        mounts: bool,
        #[structopt(short, long)]
        all: bool,
    },
}
