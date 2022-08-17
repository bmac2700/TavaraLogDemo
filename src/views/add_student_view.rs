use iced::{Alignment, Button, Column, Length, Space, Text, TextInput};

use crate::main_window::{MainView, Message};
use mysql::*;
use mysql::prelude::*;

use super::add_object_view::is_object_uid_in_use;
use super::tagfound_view::Student;

pub fn is_student_uid_in_use(uid: i64, conn: &mut PooledConn) -> bool {
    let students = conn.query_map(
        format!(
            r"SELECT * FROM itemstorage.students where uid={} LIMIT 1;",
            uid
        ),
        |(id, first_name, last_name, uid_length, uid, admin)| Student {
            id,
            first_name,
            last_name,
            uid_length,
            uid,
            admin,
        },
    ).unwrap();

    if students.len() == 1 {
        true
    }else{
        false
    }
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let first_name_input = TextInput::new(
        &mut owner.first_name_input,
        "Oppilaan etunimi",
        &owner.first_name_value,
        Message::FirstNameChanged,
    )
    .padding(15)
    .size(30)
    .width(iced::Length::Units(300));

    let last_name_input = TextInput::new(
        &mut owner.last_name_input,
        "Oppilaan sukunimi",
        &owner.last_name_value,
        Message::LastNameChanged,
    )
    .padding(15)
    .size(30)
    .width(iced::Length::Units(300));

    let mut message = if owner.new_tag.is_some() {
        let tag = owner.new_tag.clone().unwrap();
        let uid = i64::from_be_bytes(tag.uid);
        Text::new(format!("UID: {}", uid))
    } else {
        Text::new("Skannaa oppilaskortti").size(25)
    };

    let mut add_student_button =
        Button::new(&mut owner.add_student_button, Text::new("Lisää oppilas")).padding([10, 20]);

    
    let mut uid_in_use = false;

    if owner.new_tag.is_some() {
        let mut conn = owner.database_pool.get_conn().unwrap();
        uid_in_use = is_object_uid_in_use(i64::from_be_bytes(owner.new_tag.clone().unwrap().uid), &mut conn) || is_student_uid_in_use(i64::from_be_bytes(owner.new_tag.clone().unwrap().uid), &mut conn);
    }

    if owner.new_tag.is_some()
        && !owner.first_name_value.is_empty()
        && !owner.last_name_value.is_empty()
        && !uid_in_use
    {
        add_student_button = add_student_button.on_press(Message::AddStudentButton);
    }

    if uid_in_use {
        message = Text::new("RFID tagi on jo käytössä");
    }

    let back_to_main = Button::new(&mut owner.back_to_mainscreen, Text::new("Peruuta"))
        .padding([10, 20])
        .on_press(Message::BackToSettings);

    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(first_name_input)
        .push(last_name_input)
        .push(message)
        .push(add_student_button)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(back_to_main)
        .align_items(Alignment::Center);

    content
}
