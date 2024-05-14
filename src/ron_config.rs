use regex::Regex;
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
    to_string,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    error::Error,
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn save<P: AsRef<Path>, T: Debug + Serialize>(
    t: T,
    path: P,
    pretty: Option<PrettyConfig>,
) -> Result<(), Box<dyn Error>> {
    let serialized: String;
    if let Some(config) = pretty {
        serialized = to_string_pretty(&t, config)?;
    } else {
        serialized = to_string(&t)?;
    }
    let mut file = File::create(path)?;
    Ok(file.write_all(serialized.as_bytes())?)
}

pub fn parse<T: Debug + DeserializeOwned>(path: std::path) -> Result<T, Box<dyn Error>> {
    let f = fs::read(&path)?;
    let parsed: T = from_reader(&f[..])?;
    Ok(parsed)
}

pub fn trim_extension(s: &str) -> String {
    Regex::new(r"\.[^.]+$").unwrap().replace(s, "").into_owned()
}
