//=============================================================================//
//
// Tarkoitus: Tämä on asetusnäkymä, johon pääset painamalla "Asetukset" nappia päänäkymässä
//
//
//=============================================================================//

use crate::string_check;
use crate::widgets::spacer::TableSpacer;
use iced::{
    pane_grid, Alignment, Button, Color, Column, Container, Length, PickList, Scrollable, Space,
    Text, TextInput,
};
use serialport::available_ports;

use crate::widgets::style;

use crate::main_window::{MainView, Message};

use mysql::prelude::*;
use mysql::*;

use super::tagfound_view::{Object, Student};

fn get_students(conn: &mut PooledConn) -> Vec<Student> {
    conn.query_map(
        r"SELECT * FROM students;",
        |(id, first_name, last_name, group_tag, uid_length, uid, admin)| Student {
            id,
            first_name,
            last_name,
            group_tag,
            uid_length,
            uid,
            admin,
        },
    )
    .unwrap()
}

fn get_objects(conn: &mut PooledConn) -> Vec<Object> {
    conn.query_map(
        r"SELECT * FROM objects;",
        |(id, name, part_number, manufacturer, location, uid_length, uid)| Object {
            id,
            name,
            part_number,
            manufacturer,
            location,
            uid_length,
            uid,
        },
    )
    .unwrap()
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let mut devices: Vec<String> = Vec::new();

    let mut conn = owner.database_pool.get_conn().unwrap();

    let mut show_settings = false;

    if owner.teacher_tag.is_some() {
        let tag = owner.teacher_tag.clone().unwrap();

        let students = conn
            .query_map(
                format!(
                    r"SELECT * FROM students where uid={} LIMIT 1;",
                    i64::from_be_bytes(tag.uid)
                ),
                |(id, first_name, last_name, group_tag, uid_length, uid, admin)| Student {
                    id,
                    first_name,
                    last_name,
                    group_tag,
                    uid_length,
                    uid,
                    admin,
                },
            )
            .unwrap();

        if students.len() == 1 {
            let student = students[0].clone();

            if student.admin {
                show_settings = true;
            }
        }
    }

    let mut found_admin = false;

    for s in get_students(&mut conn) {
        if s.admin {
            found_admin = true;
            break;
        }
    }

    if !found_admin {
        show_settings = true;
    }

    if !show_settings && owner.selected_device.is_some() {
        let settings_button =
            Button::new(&mut owner.settings_button, Text::new("Poistu asetuksista"))
                .padding([10, 20])
                .on_press(Message::SettingsButtonClick);

        let content = Column::new()
            .push(Space::with_height(Length::FillPortion(1)))
            .push(Text::new("Skannaa opettajakortti"))
            .push(Space::with_height(Length::Units(5)))
            .push(settings_button)
            .push(Space::with_height(Length::Fill))
            .align_items(Alignment::Center);

        return content;
    }

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
        Button::new(&mut owner.add_student_view, Text::new("Lisää opiskelija"))
            .padding([10, 20])
            .on_press(Message::AddStudentViewButton);

    let add_object_view_button =
        Button::new(&mut owner.add_object_view, Text::new("Lisää työkalu"))
            .padding([10, 20])
            .on_press(Message::AddObjectViewButton);

    let students = get_students(&mut owner.database_pool.get_conn().unwrap());
    let objects = get_objects(&mut owner.database_pool.get_conn().unwrap());

    let student_search_input = TextInput::new(
        &mut owner.student_search_input,
        "Nimi",
        &owner.student_search_value,
        Message::StudentSearchChanged,
    )
    //.padding(15)
    .size(28)
    .width(iced::Length::Units(100));

    let first_row_students: iced::Row<Message> = iced::Row::new()
        .push(Text::new("ID").size(28))
        .push(Space::with_width(Length::FillPortion(3)))
        .push(student_search_input)
        .push(Space::with_width(Length::FillPortion(3)))
        .push(Text::new("Opettaja"))
        .push(Space::with_height(Length::Units(5)));
    let mut scroll_content_students = Column::new().push(first_row_students);

    for student in students {
        if !owner.student_search_value.is_empty()
            && !format!("{} {}", student.first_name, student.last_name)
                .to_lowercase()
                .starts_with(&owner.student_search_value.to_lowercase())
        {
            continue;
        }

        let teacher = if student.admin {
            Text::new("Kyllä")
        } else {
            Text::new("Ei")
        };

        let row: iced::Row<Message> = iced::Row::new()
            .push(Text::new(format!("{}", student.id)))
            .push(Space::with_width(Length::FillPortion(3)))
            .push(Text::new(format!(
                "{} {}",
                student.first_name, student.last_name
            )))
            .push(Space::with_width(Length::FillPortion(3)))
            .push(teacher);

        scroll_content_students = scroll_content_students
            .push(Space::with_height(Length::Units(5)))
            .push(row);
    }

    let student_list: Scrollable<Message> = Scrollable::new(&mut owner.student_list)
        .push(scroll_content_students)
        .height(iced::Length::Units(100))
        .width(iced::Length::Units(500));
    //-------------------------------------------

    let pane = pane_grid::PaneGrid::new(&mut owner.object_list_panes, |_id, pane| match pane.id {
        0 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Tuotteen nimi").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for object in objects.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(string_check(object.name)).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(4));

            container.into()
        }

        3 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Tuotteen tyyppi/valmistaja").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for object in objects.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(string_check(object.manufacturer)).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(4));

            container.into()
        }

        2 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Tuotteen varaosanumero").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for object in objects.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(string_check(object.part_number)).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(4));

            container.into()
        }

        1 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Sijainti/työpöytä/varasto").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for object in objects.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(string_check(object.location)).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(4));

            container.into()
        }

        _ => pane_grid::Content::new(Text::new("Jotakin meni pieleen")),
    })
    .height(Length::Units((objects.len() * 30) as u16 + 30));

    let object_list: Scrollable<Message> = Scrollable::new(&mut owner.object_list)
        .push(pane)
        .height(iced::Length::Units(150))
        .width(iced::Length::Units(1000));

    let settings_button = Button::new(&mut owner.settings_button, Text::new("Poistu asetuksista"))
        .padding([10, 20])
        .on_press(Message::SettingsButtonClick);

    let remove_student_view_button = Button::new(
        &mut owner.remove_student_view,
        Text::new("Poista opiskelija"),
    )
    .padding([10, 20])
    .on_press(Message::RemoveStudentViewButton);

    let remove_object_view_button =
        Button::new(&mut owner.remove_object_view, Text::new("Poista työkalu"))
            .padding([10, 20])
            .on_press(Message::RemoveObjectViewButton);

    let borrow_history_button = Button::new(
        &mut owner.history_button,
        Text::new("Listaa lainaushistoria"),
    )
    .padding([10, 20])
    .on_press(Message::HistoryButtonClick);

    let content = Column::new()
        .spacing(10)
        .push(Space::with_width(Length::FillPortion(1)))
        .push(device_list)
        .push(Space::with_height(Length::FillPortion(3)))
        .push(student_list)
        .push(Space::with_height(Length::Units(5)))
        .push(add_student_view_button)
        .push(remove_student_view_button)
        .push(Space::with_height(Length::Units(30)))
        .push(object_list)
        .push(Space::with_height(Length::Units(5)))
        .push(add_object_view_button)
        .push(remove_object_view_button)
        .push(Space::with_height(Length::Fill))
        .push(borrow_history_button)
        .push(settings_button)
        .align_items(Alignment::Center);

    content
}
