use iced::{Alignment, Button, Column, Length, Space, Text, TextInput};

use crate::main_window::{MainView, Message};

use mysql::*;
use mysql::prelude::*;

use super::add_student_view::is_student_uid_in_use;
use super::tagfound_view::Object;

pub fn is_object_uid_in_use(uid: i64, conn: &mut PooledConn) -> bool {
    let objects = conn.query_map(
        format!(
            r"SELECT * FROM itemstorage.objects where uid={} LIMIT 1;",
            uid
        ),
        |(id, name, uid_length, uid)| Object {
            id,
            name,
            uid_length,
            uid,
        },
    ).unwrap();

    if objects.len() == 1 {
        true
    }else{
        false
    }
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let object_name_input = TextInput::new(
        &mut owner.object_name_input,
        "Objektin nimi",
        &owner.object_name_value,
        Message::ObjectNameChanged,
    )
    .padding(15)
    .size(30)
    .width(iced::Length::Units(300));

    let mut message = if owner.new_tag.is_some() {
        let tag = owner.new_tag.clone().unwrap();
        let uid = i64::from_be_bytes(tag.uid);
        Text::new(format!("UID: {}", uid))
    } else {
        Text::new("Skannaa objekti").size(25)
    };

    let mut uid_in_use = false;

    if owner.new_tag.is_some() {
        let mut conn = owner.database_pool.get_conn().unwrap();
        uid_in_use = is_object_uid_in_use(i64::from_be_bytes(owner.new_tag.clone().unwrap().uid), &mut conn) || is_student_uid_in_use(i64::from_be_bytes(owner.new_tag.clone().unwrap().uid), &mut conn);
    }

    if uid_in_use {
        message = Text::new("RFID tagi on jo käytössä");
    }

    let mut add_object_button =
        Button::new(&mut owner.add_object_button, Text::new("Lisää objekti")).padding([10, 20]);

    if owner.new_tag.is_some() && !owner.object_name_value.is_empty() && !uid_in_use {
        add_object_button = add_object_button.on_press(Message::AddObjectButton);
    }

    let back_to_main = Button::new(&mut owner.back_to_mainscreen, Text::new("Peruuta"))
        .padding([10, 20])
        .on_press(Message::BackToSettings);

    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(object_name_input)
        .push(message)
        .push(add_object_button)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(back_to_main)
        .align_items(Alignment::Center);

    content
}
