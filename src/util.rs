use rsys::{Error, Result};
use serde::Serialize;
use serde_json as json;
use serde_yaml as yaml;
use std::any::type_name;
use std::fmt::{Debug, Display};

pub(crate) enum PrintFormat {
    Normal,
    Json,
    Yaml,
}

pub(crate) fn json_to_string<T: Serialize>(val: T, pretty: bool) -> Result<String> {
    let f = if pretty {
        json::to_string_pretty
    } else {
        json::to_string
    };

    f(&val).map_err(|e| Error::SerializeError(type_name::<T>().to_string(), e.to_string()))
}

pub(crate) fn print<T: Debug + Display + Serialize>(val: T, format: PrintFormat, pretty: bool) -> Result<()> {
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

pub(crate) fn handle_err<T: Default>(res: Result<T>) -> T {
    match res {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error: {}", e);
            T::default()
        }
    }
}

pub(crate) fn conv_b(bytes: u64) -> String {
    conv_fb(bytes as f64)
}

pub(crate) fn conv_fb(bytes: f64) -> String {
    if bytes < 1_024. {
        format!("{:.2} B", bytes)
    } else if 1_024. <= bytes && bytes < u64::pow(1_024, 2) as f64 {
        format!("{:.2} KB", bytes / 1_024.)
    } else if u64::pow(1_024, 2) as f64 <= bytes && bytes < u64::pow(1_024, 3) as f64 {
        format!("{:.2} MB", bytes / u64::pow(1_024, 2) as f64)
    } else if u64::pow(1_024, 3) as f64 <= bytes && bytes < u64::pow(1_024, 4) as f64 {
        format!("{:.2} GB", bytes / u64::pow(1_024, 3) as f64)
    } else {
        format!("{:.2} TB", bytes / u64::pow(1_024, 4) as f64)
    }
}

pub(crate) fn conv_hz(hz: u64) -> String {
    if hz < 1_000 {
        format!("{} Hz", hz)
    } else if hz < 1_000_000 {
        format!("{:.2} MHz", (hz as f64 / 1_000.0))
    } else {
        format!("{:.2} GHz", (hz as f64 / 1_000_000.0))
    }
}

pub(crate) fn conv_fhz(hz: f64) -> String {
    if hz < 1_000. {
        format!("{:.2} Hz", hz)
    } else if hz < 1_000_000. {
        format!("{:.2} MHz", (hz / 1_000.0))
    } else {
        format!("{:.2} GHz", (hz / 1_000_000.0))
    }
}
