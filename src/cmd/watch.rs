use super::WatchOpts;
use crate::{
    cli::RsysCli,
    cmd::common::SystemInfo,
    util::{print, PrintFormat},
};
use rsys::Result;
use std::{
    thread,
    time::{Duration, Instant},
};

impl RsysCli {
    pub fn watch(&self, opts: WatchOpts) -> Result<()> {
        let duration = if let Some(d) = opts.duration {
            Duration::from_secs(d)
        } else {
            Duration::from_secs(u64::MAX)
        };
        let interval: u128 = if let Some(i) = opts.interval { i as u128 } else { 1000 };
        let loop_start = Instant::now();
        loop {
            let print_start = Instant::now();
            print(
                SystemInfo::new(
                    &self.system,
                    false,
                    true,
                    false,
                    true,
                    false,
                    false,
                    opts.cpu,
                    opts.memory,
                    opts.network,
                    opts.storage,
                    false,
                    opts.all,
                    opts.stats,
                    false,
                )?,
                PrintFormat::Json,
                opts.pretty,
            )?;
            println!();
            let print_duration = print_start.elapsed().as_millis();
            if loop_start.elapsed() > duration {
                break;
            }
            if print_duration < interval {
                let sleep_duration = Duration::from_millis((interval - print_duration) as u64);
                thread::sleep(sleep_duration);
            }
        }
        Ok(())
    }
}
