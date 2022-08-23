use chrono::{DateTime, Local, NaiveDateTime, Utc};
use iced::{Alignment, Column, Length, Space, Text, TextInput};
use iced::{Button, Scrollable};

use crate::main_window::{BorrowHistoryObject, MainView, Message};

use mysql::prelude::*;

use super::main_view::{get_object_info, get_student_info};

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let mut conn = owner.database_pool.get_conn().unwrap();

    let borrows = conn
        .query_map(
                r"SELECT * FROM borrow_history where borrow_end_timestamp is not null and student_id is not null and object_id is not null order by borrow_end_timestamp desc;",
            |(id, student_id, object_id, borrow_start_timestamp, borrow_end_timestamp)| BorrowHistoryObject {
                id,
                student_id,
                object_id,
                borrow_start_timestamp,
                borrow_end_timestamp,
            },
        )
        .unwrap();

    let student_search_input = TextInput::new(
        &mut owner.student_search_input,
        "Lainaaja",
        &owner.student_search_value,
        Message::StudentSearchChanged,
    )
    //.padding(15)
    .size(28)
    .width(iced::Length::Units(100));

    let first_row_history: iced::Row<Message> = iced::Row::new()
        //.push(Text::new("Lainaaja").size(28))
        .push(student_search_input)
        .push(Space::with_width(Length::FillPortion(3)))
        .push(Text::new("Esine").size(28))
        .push(Space::with_width(Length::FillPortion(3)))
        .push(Text::new("Lainaus aika").size(28))
        .push(Space::with_width(Length::FillPortion(3)))
        .push(Text::new("Palautus aika").size(28))
        .push(Space::with_height(Length::Units(5)));
    let mut scroll_content_history = Column::new().push(first_row_history);

    for x in borrows {
        let object = get_object_info(x.object_id, &mut conn).unwrap();
        let student = get_student_info(x.student_id, &mut conn).unwrap();

        //Filters

        if !owner.student_search_value.is_empty()
            && !format!("{} {}", student.first_name, student.last_name)
                .to_lowercase()
                .starts_with(&owner.student_search_value.to_lowercase())
        {
            continue;
        }

        //Filters

        let naive = NaiveDateTime::from_timestamp(x.borrow_start_timestamp, 0);
        let borrow_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let borrow_time = borrow_time.with_timezone(&Local);
        let borrow_time_formatted = borrow_time.format("%H:%M:%S %d/%m/%Y");

        let naive = NaiveDateTime::from_timestamp(x.borrow_end_timestamp.unwrap(), 0);
        let return_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let return_time = return_time.with_timezone(&Local);
        let return_time_formatted = return_time.format("%H:%M:%S %d/%m/%Y");

        let row: iced::Row<Message> = iced::Row::new()
            .push(Text::new(format!(
                "{} {}",
                student.first_name, student.last_name
            )))
            .push(Space::with_width(Length::FillPortion(3)))
            .push(Text::new(object.name))
            .push(Space::with_width(Length::FillPortion(3)))
            .push(Text::new(format!("{}", borrow_time_formatted)))
            .push(Space::with_width(Length::FillPortion(3)))
            .push(Text::new(format!("{}", return_time_formatted)));

        scroll_content_history = scroll_content_history
            .push(Space::with_height(Length::Units(5)))
            .push(row);
    }

    let borrow_list: Scrollable<Message> = Scrollable::new(&mut owner.borrow_history)
        .push(scroll_content_history)
        .height(iced::Length::Units(200))
        .width(iced::Length::Units(600));

    let back_to_main = Button::new(&mut owner.back_to_mainscreen, Text::new("Peruuta"))
        .padding([10, 20])
        .on_press(Message::BackToSettings);

    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(Text::new("Lainaus historia").size(32))
        .push(borrow_list)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(back_to_main)
        .align_items(Alignment::Center);

    content
}
