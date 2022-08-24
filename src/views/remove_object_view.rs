use iced::{Alignment, Button, Column, Length, Space, Text, TextInput};

use crate::main_window::{MainView, Message};

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let object_id_input = TextInput::new(
        &mut owner.object_id_input,
        "Työkalun ID",
        &owner.object_id_value,
        Message::ObjectIdChanged,
    )
    .padding(15)
    .size(30)
    .width(iced::Length::Units(300));

    let mut remove_object_button =
        Button::new(&mut owner.remove_object_button, Text::new("Poista työkalu"));

    if !owner.object_id_value.is_empty() {
        remove_object_button = remove_object_button.on_press(Message::RemoveObjectButton);
    }

    let back_to_main = Button::new(&mut owner.back_to_mainscreen, Text::new("Peruuta"))
        .padding([10, 20])
        .on_press(Message::BackToSettings);

    let content = Column::new()
        .push(Space::with_height(Length::Units(300)))
        .push(object_id_input)
        .push(remove_object_button)
        .push(Space::with_height(Length::Units(10)))
        .push(back_to_main)
        .align_items(Alignment::Center);

    content
}
