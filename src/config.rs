use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use toml::Value;

pub fn load_config(path: &Path) -> Result<HashMap<String, Value>, io::Error> {
    let content = fs::read_to_string(path)?;

    let parsed: toml::Table = match content.parse() {
        Ok(value) => value,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse TOML: {}", err),
            ));
        }
    };

    let config: HashMap<String, Value> = parsed.into_iter().collect();

    Ok(config)
}
