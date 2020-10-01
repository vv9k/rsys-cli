use rsys::{Result, Rsys};
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
        #[structopt()]
        property: String,
    },
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
                RsysCmd::Get { property: prop } => match prop.as_str() {
                    "hostname" => print!("{}", self.system.hostname()?),
                    "uptime" => print!("{}", self.system.uptime()?),
                    "os" => print!("{}", self.system.os()),
                    "arch" => print!("{}", self.system.arch()?),
                    "cpu" => print!("{}", self.system.cpu()?),
                    "cpu_clock" => print!("{}", self.system.cpu_clock()?),
                    "cpu_cores" => print!("{}", self.system.cpu_cores()?),
                    "logical_cores" => print!("{}", self.system.logical_cores()?),
                    "memory_total" => print!("{}", self.system.memory_total()?),
                    "memory_free" => print!("{}", self.system.memory_free()?),
                    "swap_total" => print!("{}", self.system.swap_total()?),
                    "swap_free" => print!("{}", self.system.swap_free()?),
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let rsys = RsysCli::new();
    rsys.main()?;

    Ok(())
}
