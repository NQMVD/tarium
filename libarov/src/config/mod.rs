pub mod filters;
pub mod mod_state;
pub mod structs;

use log::{debug, info};
use std::{
    fs::{create_dir_all, File},
    io::{BufReader, Result},
    path::Path,
};

/// Open the config file at `path` and deserialise it into a config struct
pub fn read_config(path: impl AsRef<Path>) -> Result<structs::Config> {
    if !path.as_ref().exists() {
        create_dir_all(path.as_ref().parent().expect("Invalid config directory"))?;
        write_config(&path, &structs::Config::default())?;
    }

    debug!(SCOPE = "libarov::config", path:debug = &path.as_ref(); "opening config file");
    let config_file = BufReader::new(File::open(&path)?);

    let mut config: structs::Config = serde_json::from_reader(config_file)?;
    info!(SCOPE = "libarov::config"; "config deserialised");

    // config
    //     .profiles
    //     .iter_mut()
    //     .for_each(structs::Profile::backwards_compat);

    Ok(config)
}

pub fn write_config(path: impl AsRef<Path>, config: &structs::Config) -> Result<()> {
    info!(SCOPE = "libarov::config", path:debug = &path.as_ref(); "writing config");
    let config_file = File::create(path)?;

    serde_json::to_writer_pretty(config_file, config)?;

    info!(SCOPE = "libarov::config"; "config write complete");
    Ok(())
}
