use iced::{Button, Column, Length, Scrollable, Space, Text};
use mysql::prelude::*;
use mysql::*;

use crate::main_window::{BorrowHistoryObject, MainView, Message};

use super::tagfound_view::{Object, Student};

pub fn get_student_info(id: i64, conn: &mut PooledConn) -> Option<Student> {
    let students = conn
        .query_map(
            format!(r"SELECT * FROM students where id={} LIMIT 1;", id),
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
            |(id, name, uid_length, uid)| Object {
                id,
                name,
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
    pub first_name: String,
    pub last_name: String,
    pub item_name: String,
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

        let info = BorrowInfo {
            first_name: student.first_name,
            last_name: student.last_name,
            item_name: object.name,
        };

        borrow_info.push(info);
    }

    borrow_info
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let settings_button = Button::new(&mut owner.settings_button, Text::new("Asetukset"))
        .padding([10, 20])
        .on_press(Message::SettingsButtonClick);

    let scan_message = Text::new("Skannaa oppilaskortti").size(32);

    let first_row: iced::Row<Message> = iced::Row::new()
        .push(Text::new("Oppilas").size(28))
        .push(Space::with_width(Length::FillPortion(50)))
        .push(Text::new("Esine").size(28))
        .push(Space::with_height(Length::Units(5)));
    let mut scroll_content = Column::new().push(first_row);

    let borrowed_items = get_borrowed_items(&mut owner.database_pool.get_conn().unwrap());

    for x in borrowed_items {
        let row: iced::Row<Message> = iced::Row::new()
            .push(Text::new(format!("{} {}", x.first_name, x.last_name)))
            .push(Space::with_width(Length::FillPortion(50)))
            .push(Text::new(x.item_name));
        scroll_content = scroll_content
            .push(Space::with_height(Length::Units(5)))
            .push(row);
    }

    let borrow_list: Scrollable<Message> = Scrollable::new(&mut owner.borrow_list)
        .push(scroll_content)
        .height(iced::Length::Units(100))
        .width(iced::Length::Units(350));

    let content = Column::new()
        .push(Text::new("Työkalujen lainaus järjestelmä").size(16))
        .push(Space::with_height(Length::FillPortion(1)))
        .push(scan_message)
        .push(Space::with_height(Length::Units(50)))
        .push(borrow_list)
        .push(Space::with_height(Length::Fill))
        .push(settings_button)
        .align_items(iced::Alignment::Center);

    content
}
