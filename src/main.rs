//=============================================================================//
//
// Tarkoitus: Sisältää ensimmäisen funktion joka käynnistyy. Tiedosto sisältää
// konfiguraatio tiedoston luonnin ja pääikkunan avaamisen.
//
//
//=============================================================================//

use crate::{
    beep::beep,
    configuration_file::{write_configuration, MainConfigurationFile},
};

mod beep;
mod configuration_file;
mod main_window;
mod scanner;
mod views;
mod widgets;

pub fn string_check(x: String) -> String {
    if x.len() == 0 {
        return " ".into();
    }
    x
}
fn main() {
    beep(0.0, std::time::Duration::from_millis(0));
    if let Err(_) = configuration_file::read_configuration("./config.toml") {
        write_configuration(
            "./config.toml",
            &MainConfigurationFile {
                database_url: "mysql://root:qwe321.@localhost:3306/itemstorage".into(),
                ..Default::default()
            },
        );
    }

    main_window::launch().unwrap();
}
