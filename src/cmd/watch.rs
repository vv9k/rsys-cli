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
    pub(crate) fn watch(
        &self,
        pretty: bool,
        cpu: bool,
        memory: bool,
        net: bool,
        storage: bool,
        all: bool,
        stats: bool,
        duration: Option<u64>,
        interval: Option<u64>,
    ) -> Result<()> {
        let duration = if let Some(d) = duration {
            Duration::from_secs(d)
        } else {
            Duration::from_secs(u64::MAX)
        };
        let interval: u128 = if let Some(i) = interval { i as u128 } else { 1000 };
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
                    cpu,
                    memory,
                    net,
                    storage,
                    false,
                    all,
                    stats,
                )?,
                PrintFormat::Json,
                pretty,
            )?;
            print!("\n");
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
