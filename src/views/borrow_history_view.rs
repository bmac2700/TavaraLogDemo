//=============================================================================//
//
// Tarkoitus: Sisältää lainaushistoria näkymän
//
//
//=============================================================================//

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use iced::{pane_grid, Alignment, Color, Column, Container, Length, Row, Space, Text, TextInput};
use iced::{Button, Scrollable};

use crate::main_window::{BorrowHistoryObject, MainView, Message};
use crate::widgets::spacer::TableSpacer;
use crate::widgets::style;

use mysql::prelude::*;

use super::main_view::{get_object_info, get_student_info};

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let mut conn = owner.database_pool.get_conn().unwrap();

    let mut borrows = conn
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
    .size(22)
    .width(iced::Length::Units(150));

    let object_search_input = TextInput::new(
        &mut owner.object_search_input,
        "Työkalu",
        &owner.object_search_value,
        Message::ObjectFilterChanged,
    )
    //.padding(15)
    .size(22)
    .width(iced::Length::Units(150));

    //Filters
    borrows.retain(|f| {
        let student =
            get_student_info(f.student_id, &mut owner.database_pool.get_conn().unwrap()).unwrap();
        let object =
            get_object_info(f.object_id, &mut owner.database_pool.get_conn().unwrap()).unwrap();

        if !owner.student_search_value.is_empty()
            && !format!("{} {}", student.first_name, student.last_name)
                .to_lowercase()
                .starts_with(&owner.student_search_value.to_lowercase())
        {
            return false;
        }

        if !owner.object_search_value.is_empty()
            && !object
                .name
                .to_lowercase()
                .starts_with(&owner.object_search_value)
        {
            return false;
        }

        true
    });

    //Filters

    let pane =
        pane_grid::PaneGrid::new(&mut owner.borrow_history_panes, |_id, pane| match pane.id {
            0 => {
                let mut content: Column<Message> = Column::new()
                    .spacing(5)
                    .push(
                        iced::Row::new()
                            .push(Space::with_width(Length::Units(5)))
                            .push(Text::new("Työkalu").size(22)),
                    )
                    .push(TableSpacer::new(1f32, Color::BLACK));

                for borrow in borrows.clone() {
                    let object = get_object_info(
                        borrow.object_id,
                        &mut owner.database_pool.get_conn().unwrap(),
                    )
                    .unwrap();

                    content = content
                        .push(
                            iced::Row::new()
                                .push(Space::with_width(Length::Units(2)))
                                .push(Text::new(object.name).size(18)),
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
                            .push(Text::new("Lainaaja").size(22)),
                    )
                    .push(TableSpacer::new(1f32, Color::BLACK));

                for borrow in borrows.clone() {
                    let student = get_student_info(
                        borrow.student_id,
                        &mut owner.database_pool.get_conn().unwrap(),
                    )
                    .unwrap();
                    content = content
                        .push(
                            iced::Row::new()
                                .push(Space::with_width(Length::Units(2)))
                                .push(
                                    Text::new(format!(
                                        "{} {}",
                                        student.first_name, student.last_name
                                    ))
                                    .size(18),
                                ),
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
                            .push(Text::new("Lainaus aika").size(22)),
                    )
                    .push(TableSpacer::new(1f32, Color::BLACK));

                for borrow in borrows.clone() {
                    let naive = NaiveDateTime::from_timestamp(borrow.borrow_start_timestamp, 0);
                    let borrow_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                    let borrow_time = borrow_time.with_timezone(&Local);
                    let borrow_time_formatted = borrow_time.format("%H:%M:%S %d/%m/%Y");
                    content = content
                        .push(
                            iced::Row::new()
                                .push(Space::with_width(Length::Units(2)))
                                .push(Text::new(format!("{}", borrow_time_formatted)).size(18)),
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
                            .push(Text::new("Palautus aika").size(22)),
                    )
                    .push(TableSpacer::new(1f32, Color::BLACK));

                for borrow in borrows.clone() {
                    let naive =
                        NaiveDateTime::from_timestamp(borrow.borrow_end_timestamp.unwrap(), 0);
                    let return_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                    let return_time = return_time.with_timezone(&Local);
                    let return_time_formatted = return_time.format("%H:%M:%S %d/%m/%Y");
                    content = content
                        .push(
                            iced::Row::new()
                                .push(Space::with_width(Length::Units(2)))
                                .push(Text::new(format!("{}", return_time_formatted)).size(18)),
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
        .height(Length::Units((borrows.len() * 30) as u16 + 30));

    let borrow_history: Scrollable<Message> = Scrollable::new(&mut owner.borrow_history)
        .push(pane)
        .height(iced::Length::Units(300))
        .width(iced::Length::Units(1200));

    let back_to_main = Button::new(&mut owner.back_to_mainscreen, Text::new("Peruuta"))
        .padding([10, 20])
        .on_press(Message::BackToSettings);

    let filter_row = Row::new()
        .push(object_search_input)
        .push(Space::with_width(Length::FillPortion(4)))
        .push(student_search_input)
        .push(Space::with_width(Length::FillPortion(4)))
        .push(Text::new("Lainaus aika").size(22))
        .push(Space::with_width(Length::FillPortion(4)))
        .push(Text::new("Palautus aika").size(22))
        .width(iced::Length::Units(1200));

    let content = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(Text::new("Suodattimet"))
        .push(filter_row)
        .push(Text::new("Lainaus historia").size(32))
        .push(borrow_history)
        .push(Space::with_height(Length::FillPortion(25)))
        .push(back_to_main)
        .align_items(Alignment::Center);

    content
}
