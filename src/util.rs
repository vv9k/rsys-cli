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
