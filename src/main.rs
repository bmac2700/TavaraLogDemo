use crate::configuration_file::{write_configuration, MainConfigurationFile};

mod configuration_file;
mod main_window;
mod scanner;
mod views;

fn main() {
    println!("Hello, world!");

    match configuration_file::read_configuration("./config.toml") {
        Err(_) => {
            write_configuration(
                "./config.toml",
                &MainConfigurationFile {
                    database_url: "mysql://root:qwe321.@localhost:3306/itemstorage".into(),
                    ..Default::default()
                },
            );
        }
        Ok(_) => {}
    }

    main_window::launch().unwrap();
}
