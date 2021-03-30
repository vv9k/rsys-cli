use rand::seq::IteratorRandom;
use rsys::{Error, Result};
use serde::Serialize;
use serde_json as json;
use serde_yaml as yaml;
use std::any::type_name;
use std::fmt::{Debug, Display};
use tui::style::Color;

const KILO: f64 = 1000.;
const MEGA: f64 = KILO * KILO;
const GIGA: f64 = KILO * KILO * KILO;
const TERA: f64 = KILO * KILO * KILO * KILO;

pub enum PrintFormat {
    Normal,
    Json,
    Yaml,
}
impl PrintFormat {
    pub fn from_bools(json: bool, yaml: bool) -> Self {
        if json {
            PrintFormat::Json
        } else if yaml {
            PrintFormat::Yaml
        } else {
            PrintFormat::Normal
        }
    }
}

pub fn json_to_string<T: Serialize>(val: T, pretty: bool) -> Result<String> {
    let f = if pretty {
        json::to_string_pretty
    } else {
        json::to_string
    };

    f(&val).map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))
}

pub fn print<T: Debug + Display + Serialize>(val: T, format: PrintFormat, pretty: bool) -> Result<()> {
    match format {
        PrintFormat::Normal => {
            if pretty {
                print!("{:#?}", val);
            } else {
                print!("{}", val);
            }
        }
        PrintFormat::Json => {
            print!("{}", json_to_string(val, pretty)?);
        }
        PrintFormat::Yaml => {
            print!(
                "{}",
                yaml::to_string(&val)
                    .map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))?
            );
        }
    }

    Ok(())
}

pub fn handle_err<T: Default>(res: Result<T>) -> T {
    match res {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error: {}", e);
            T::default()
        }
    }
}

fn conv_metric(value: f64, unit: &str) -> String {
    let (val, u) = if value < KILO {
        (value, "")
    } else if KILO <= value && value < MEGA {
        (value / KILO, "K")
    } else if MEGA <= value && value < GIGA {
        (value / MEGA, "M")
    } else if GIGA <= value && value < TERA {
        (value / GIGA, "G")
    } else {
        (value / TERA, "T")
    };

    format!("{:.2}{}{}", val, u, unit)
}

pub fn conv_fbs(bytes: f64) -> String {
    conv_metric(bytes, "B/s")
}

pub fn conv_fb(bytes: f64) -> String {
    conv_metric(bytes, "B")
}

pub fn conv_b(bytes: u64) -> String {
    conv_fb(bytes as f64)
}

pub fn conv_hz(hz: u64) -> String {
    conv_fhz(hz as f64)
}

pub fn conv_fhz(hz: f64) -> String {
    conv_metric(hz, "Hz")
}

pub fn conv_t(time: f64) -> String {
    format!("{:.1}s", time)
}

pub fn conv_p(val: f64) -> String {
    format!("{:.1}%", val)
}

pub fn random_color(min: Option<u8>) -> Color {
    let mut rng = rand::thread_rng();
    let mut color: [u8; 3] = [0, 0, 0];

    let low = if let Some(min) = min { min } else { 0 };

    (low..255).choose_multiple_fill(&mut rng, &mut color);
    Color::Rgb(color[0], color[1], color[2])
}
