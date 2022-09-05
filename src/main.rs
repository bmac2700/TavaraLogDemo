//=============================================================================//
//
// Tarkoitus: Sisältää ensimmäisen funktion joka käynnistyy. Tiedosto sisältää
// konfiguraatio tiedoston luonnin ja pääikkunan avaamisen.
//
//
//=============================================================================//

use crate::configuration_file::{write_configuration, MainConfigurationFile};

mod beep;
mod configuration_file;
mod main_window;
mod scanner;
mod views;

fn main() {
    println!("Hello, world!");

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
