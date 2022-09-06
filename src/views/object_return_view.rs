//=============================================================================//
//
// Tarkoitus: Sisältää työkalun palautus näkymän, joka avautuu kun työkalu palautetaan
//
//
//=============================================================================//

use iced::{Alignment, Column, Length, Space, Text};

use crate::main_window::{MainView, Message};

pub fn get_view(_owner: &mut MainView) -> Column<Message> {
    std::thread::spawn(|| {
        crate::beep::beep(1250.0, std::time::Duration::from_millis(60));
        std::thread::sleep(std::time::Duration::from_millis(10));
        crate::beep::beep(1250.0, std::time::Duration::from_millis(60));
    });

    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(Text::new("Työkalu palautettu").size(32))
        .push(Space::with_height(Length::FillPortion(25)))
        .align_items(Alignment::Center);

    content
}
