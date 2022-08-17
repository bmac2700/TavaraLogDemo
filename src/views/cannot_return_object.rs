use iced::{Alignment, Column, Length, Space, Text};

use crate::main_window::{MainView, Message};

pub fn get_view(_owner: &mut MainView) -> Column<Message> {
    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(Text::new("Et voi palauttaa esinettä, sillä lainasit sen juuri").size(32))
        .push(Space::with_height(Length::FillPortion(25)))
        .align_items(Alignment::Center);

    content
}
