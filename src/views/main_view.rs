//=============================================================================//
//
// Tarkoitus: Tämä tiedosto sisältää itse päänäkymän.
//
//
//=============================================================================//

use crate::widgets::spacer::TableSpacer;
use crate::widgets::style;
use iced::{pane_grid, Button, Color, Column, Container, Length, Scrollable, Space, Text};
use mysql::prelude::*;
use mysql::*;

use crate::main_window::{BorrowHistoryObject, MainView, Message};

use super::tagfound_view::{Object, Student};

pub fn get_student_info(id: i64, conn: &mut PooledConn) -> Option<Student> {
    let students = conn
        .query_map(
            format!(r"SELECT * FROM students where id={} LIMIT 1;", id),
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
        Some(students[0].clone())
    } else {
        None
    }
}

pub fn get_object_info(id: i64, conn: &mut PooledConn) -> Option<Object> {
    let objects = conn
        .query_map(
            format!(r"SELECT * FROM objects where id={} LIMIT 1;", id),
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
        .unwrap();

    if objects.len() == 1 {
        Some(objects[0].clone())
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct BorrowInfo {
    pub student: Student,
    pub object: Object,
}

fn get_borrowed_items(conn: &mut PooledConn) -> Vec<BorrowInfo> {
    let mut borrow_info: Vec<BorrowInfo> = Vec::new();

    let borrows = conn
        .query_map(
                r"SELECT * FROM borrow_history where borrow_end_timestamp is null and student_id is not null and object_id is not null;",
            |(id, student_id, object_id, borrow_start_timestamp, borrow_end_timestamp)| BorrowHistoryObject {
                id,
                student_id,
                object_id,
                borrow_start_timestamp,
                borrow_end_timestamp,
            },
        )
        .unwrap();

    for in_borrow_item in borrows {
        let student = get_student_info(in_borrow_item.student_id, conn).unwrap();
        let object = get_object_info(in_borrow_item.object_id, conn).unwrap();

        let info = BorrowInfo { student, object };

        borrow_info.push(info);
    }

    borrow_info
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let settings_button = Button::new(&mut owner.settings_button, Text::new("Asetukset"))
        .padding([5, 10])
        .on_press(Message::SettingsButtonClick);

    let scan_message = Text::new("Skannaa opiskelijan kortti").size(32);
    let scan_message_down = Text::new("Tai palauta esine skannamaalla se").size(16);

    let borrowed_items = get_borrowed_items(&mut owner.database_pool.get_conn().unwrap());

    let pane = pane_grid::PaneGrid::new(&mut owner.borrow_list_panes, |_id, pane| match pane.id {
        0 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Tuotteen nimi").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for borrowed_item in borrowed_items.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(borrowed_item.object.name).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(3));

            container.into()
        }

        1 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Lainaaja").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for borrowed_item in borrowed_items.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(
                                Text::new(format!(
                                    "{}, {}, {}",
                                    borrowed_item.student.last_name,
                                    borrowed_item.student.first_name,
                                    borrowed_item.student.group_tag
                                ))
                                .size(18),
                            ),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(3));

            container.into()
        }

        2 => {
            let mut content: Column<Message> = Column::new()
                .spacing(5)
                .push(
                    iced::Row::new()
                        .push(Space::with_width(Length::Units(5)))
                        .push(Text::new("Palautuspaikka").size(22)),
                )
                .push(TableSpacer::new(1f32, Color::BLACK));

            for borrowed_item in borrowed_items.clone() {
                content = content
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(2)))
                            .push(Text::new(borrowed_item.object.location).size(18)),
                    )
                    .push(TableSpacer::new(1f32, Color::from_rgb(0.75, 0.75, 0.75)));
            }

            let container = Container::new(content)
                .style(style::Theme::Primary)
                .width(Length::FillPortion(3));

            container.into()
        }

        _ => pane_grid::Content::new(Text::new("Jotakin meni pieleen")),
    })
    .height(Length::Units((borrowed_items.len() * 30) as u16 + 30));

    let borrow_list: Scrollable<Message> = Scrollable::new(&mut owner.borrow_list)
        .push(pane)
        .height(iced::Length::Units(150))
        .width(iced::Length::Units(800));

    let content = Column::new()
        .push(Text::new("Työkalujen lainaus järjestelmä").size(16))
        .push(Space::with_height(Length::FillPortion(1)))
        .push(scan_message)
        .push(scan_message_down)
        .push(Space::with_height(Length::Units(50)))
        .push(borrow_list)
        .push(Space::with_height(Length::Fill))
        .push(settings_button)
        .align_items(iced::Alignment::Center);

    content
}
