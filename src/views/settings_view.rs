use iced::{Alignment, Button, Column, Length, PickList, Scrollable, Space, Text};
use serialport::available_ports;

use crate::main_window::{MainView, Message};

use mysql::prelude::*;
use mysql::*;

use super::tagfound_view::{Object, Student};

fn get_students(conn: &mut PooledConn) -> Vec<Student> {
    let students = conn
        .query_map(
            r"SELECT * FROM itemstorage.students;",
            |(id, first_name, last_name, uid_length, uid, admin)| Student {
                id,
                first_name,
                last_name,
                uid_length,
                uid,
                admin,
            },
        )
        .unwrap();

    students
}

fn get_objects(conn: &mut PooledConn) -> Vec<Object> {
    let objects = conn
        .query_map(
            r"SELECT * FROM itemstorage.objects;",
            |(id, name, uid_length, uid)| Object {
                id,
                name,
                uid_length,
                uid,
            },
        )
        .unwrap();

    objects
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let mut devices: Vec<String> = Vec::new();

    let ports = available_ports().unwrap();

    for port in ports {
        let device_name = match port.port_type {
            serialport::SerialPortType::UsbPort(x) => {
                if x.product.is_some() {
                    x.product.unwrap()
                } else {
                    "Unknown device".to_string()
                }
            }
            _ => "Unknown device".to_string(),
        };
        devices.push(format!("{} - {}", port.port_name, device_name));
    }

    let device_list = PickList::new(
        &mut owner.device_list,
        devices,
        owner.selected_device.clone(),
        Message::DeviceSelected,
    )
    .placeholder("Valitse USB lukija");

    let add_student_view_button =
        Button::new(&mut owner.add_student_view, Text::new("Lisää oppilas"))
            .padding([10, 20])
            .on_press(Message::AddStudentViewButton);

    let add_object_view_button =
        Button::new(&mut owner.add_object_view, Text::new("Lisää objekti"))
            .padding([10, 20])
            .on_press(Message::AddObjectViewButton);

    let students = get_students(&mut owner.database_pool.get_conn().unwrap());
    let objects = get_objects(&mut owner.database_pool.get_conn().unwrap());

    let first_row_students: iced::Row<Message> = iced::Row::new()
        .push(Text::new("ID").size(28))
        .push(Space::with_width(Length::FillPortion(25)))
        .push(Text::new("Nimi").size(28))
        .push(Space::with_width(Length::FillPortion(25)))
        .push(Text::new("UID").size(28))
        .push(Space::with_height(Length::Units(5)));
    let mut scroll_content_students = Column::new().push(first_row_students);

    for x in students {
        let row: iced::Row<Message> = iced::Row::new()
            .push(Text::new(format!("{}", x.id)))
            .push(Space::with_width(Length::FillPortion(25)))
            .push(Text::new(format!("{} {}", x.first_name, x.last_name)))
            .push(Space::with_width(Length::FillPortion(25)))
            .push(Text::new(format!("{}", x.uid)));
        //.push(Button::new(&mut owner.add_object_button, Text::new("Epic")).on_press(Message::EditStudent(x.id)));

        scroll_content_students = scroll_content_students
            .push(Space::with_height(Length::Units(5)))
            .push(row);
    }

    let student_list: Scrollable<Message> = Scrollable::new(&mut owner.student_list)
        .push(scroll_content_students)
        .height(iced::Length::Units(100))
        .width(iced::Length::Units(500));
    //-------------------------------------------
    let first_row_objects: iced::Row<Message> = iced::Row::new()
        .push(Text::new("ID").size(28))
        .push(Space::with_width(Length::FillPortion(25)))
        .push(Text::new("Esine").size(28))
        .push(Space::with_width(Length::FillPortion(25)))
        .push(Text::new("UID").size(28))
        .push(Space::with_height(Length::Units(5)));
    let mut scroll_content_objects = Column::new().push(first_row_objects);

    for x in objects {
        let row: iced::Row<Message> = iced::Row::new()
            .push(Text::new(format!("{}", x.id)))
            .push(Space::with_width(Length::FillPortion(25)))
            .push(Text::new(format!("{}", x.name)))
            .push(Space::with_width(Length::FillPortion(25)))
            .push(Text::new(format!("{}", x.uid)));
        //.push(Button::new(&mut owner.add_object_button, Text::new("Epic")).on_press(Message::EditStudent(x.id)));

        scroll_content_objects = scroll_content_objects
            .push(Space::with_height(Length::Units(5)))
            .push(row);
    }

    let object_list: Scrollable<Message> = Scrollable::new(&mut owner.object_list)
        .push(scroll_content_objects)
        .height(iced::Length::Units(100))
        .width(iced::Length::Units(500));

    let settings_button = Button::new(&mut owner.settings_button, Text::new("Poistu asetuksista"))
        .padding([10, 20])
        .on_press(Message::SettingsButtonClick);

    let remove_student_view_button =
        Button::new(&mut owner.remove_student_view, Text::new("Poista oppilas"))
            .padding([10, 20])
            .on_press(Message::RemoveStudentViewButton);

    let remove_object_view_button =
        Button::new(&mut owner.remove_object_view, Text::new("Poista objekti"))
            .padding([10, 20])
            .on_press(Message::RemoveObjectViewButton);

    let content = Column::new()
        .spacing(10)
        .push(Space::with_width(Length::FillPortion(1)))
        .push(device_list)
        .push(Space::with_height(Length::FillPortion(3)))
        .push(student_list)
        .push(Space::with_height(Length::Units(5)))
        .push(add_student_view_button)
        .push(remove_student_view_button)
        .push(Space::with_height(Length::FillPortion(3)))
        .push(object_list)
        .push(Space::with_height(Length::Units(5)))
        .push(add_object_view_button)
        .push(remove_object_view_button)
        .push(Space::with_height(Length::Fill))
        .push(settings_button)
        .align_items(Alignment::Center);

    content
}
