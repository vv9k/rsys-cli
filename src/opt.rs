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
}
