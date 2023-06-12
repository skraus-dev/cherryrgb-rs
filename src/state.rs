use anyhow::{anyhow, Error};
use cherryrgb::CustomKeyLeds;
use std::{fs::File, io::BufReader, path::PathBuf};

const STATEFILE: &str = "cherryrgb_state.json";

/// Return the state directory
fn state_dir() -> Result<PathBuf, Error> {
    match dirs::cache_dir() {
        Some(path) => Ok(path),
        None => Err(anyhow!("Could not get cache directory")),
    }
}

/// Save custom colors to file
pub fn save(leds: CustomKeyLeds) -> Result<(), Error> {
    let dir = state_dir()?;
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(STATEFILE);
    log::debug!("Saving state to {file:?}");
    serde_json::to_writer(&File::create(file)?, &leds)?;
    Ok(())
}

/// Read custom colors from file
fn read_state() -> Result<CustomKeyLeds, Error> {
    let path = state_dir()?.join(STATEFILE);
    let file = File::open(path.clone())?;
    let reader = BufReader::new(file);
    let ret = serde_json::from_reader(reader)?;
    log::debug!("Loaded state from {path:?}");
    Ok(ret)
}

/// Load custom colors from file or create a ne instance
pub fn load() -> Result<CustomKeyLeds, Error> {
    if let Ok(ret) = read_state() {
        return Ok(ret);
    }
    let ret = CustomKeyLeds::new();
    Ok(ret)
}
