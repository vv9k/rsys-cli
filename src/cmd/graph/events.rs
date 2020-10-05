use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use termion::{event::Key, input::TermRead};

pub(crate) const DEFAULT_TICK_RATE: u64 = 1000;

#[derive(Debug)]
pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Debug)]
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}
impl Config {
    pub fn new(tick_rate: u64) -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(tick_rate),
        }
    }

    pub fn new_or_default(tick_rate: Option<u64>) -> Config {
        let tick_rate = if let Some(t) = tick_rate { t } else { DEFAULT_TICK_RATE };
        Config::new(tick_rate)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(300),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let _ = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(_) = tx.send(Event::Input(key)) {
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let _ = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
