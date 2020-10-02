use super::get::Property;
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
