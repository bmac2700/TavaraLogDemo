//=============================================================================//
//
// Tarkoitus: Sisältää kaikki konfiguraation lukemisen ja kirjoittamisen levylle.
//
//
//=============================================================================//

use std::io::Write;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct MainConfigurationFile {
    pub database_url: String,
    pub scanner_port: String,
    pub scanner_port_name: String,
}

pub fn read_configuration(filename: &str) -> Result<MainConfigurationFile, std::io::Error> {
    let file_contents = std::fs::read_to_string(filename)?;

    let result: MainConfigurationFile = toml::from_str(&file_contents)?;

    Ok(result)
}

pub fn write_configuration(filename: &str, config: &MainConfigurationFile) -> bool {
    let encoded_config = match toml::to_string_pretty(config) {
        Err(_) => {
            return false;
        }
        Ok(v) => v,
    };

    let mut file = std::fs::File::create(filename).unwrap();

    match file.write(encoded_config.as_bytes()) {
        Err(_) => false,
        Ok(_) => true,
    }
}
