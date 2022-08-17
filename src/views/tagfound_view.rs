use iced::{Alignment, Column, Length, Space, Text};

use crate::main_window::{is_object_borrowed, MainView, Message};

use mysql::{params, prelude::*};

#[derive(Debug, Clone)]
pub struct Student {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub uid_length: u8,
    pub uid: i64,
    pub admin: bool,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub id: i64,
    pub name: String,
    pub uid_length: u8,
    pub uid: i64,
}

pub fn get_view(owner: &mut MainView) -> Column<Message> {
    let student_tag = owner.last_scanned_student_tag.as_ref().unwrap().clone();

    let student_uid = i64::from_be_bytes(student_tag.uid);

    let mut conn = owner.database_pool.get_conn().unwrap();

    //Ei ole hyvä tapa, mahdollinen sql injection, mutta todella epätodennäköinen sillä tagin uid pitäisi olla tekstiä, mutta jos se olisi aika varmasti arduino luulee sitä vialliseksi eikä lähetä sitä
    let students = conn.query_map(
        format!(
            r"SELECT * FROM itemstorage.students where uid={} LIMIT 1;",
            student_uid
        ),
        |(id, first_name, last_name, uid_length, uid, admin)| Student {
            id,
            first_name,
            last_name,
            uid_length,
            uid,
            admin,
        },
    );

    let student = students.unwrap()[0].clone();

    let mut object: Option<Object> = None;

    let message = if owner.last_scanned_object_tag.is_some() {
        let object_tag = owner.last_scanned_object_tag.as_ref().unwrap().clone();
        let object_uid = i64::from_be_bytes(object_tag.uid);

        let objects = conn.query_map(
            format!(
                r"SELECT * FROM itemstorage.objects where uid={} LIMIT 1;",
                object_uid
            ),
            |(id, name, uid_length, uid)| Object {
                id,
                name,
                uid_length,
                uid,
            },
        );

        let db_object = objects.unwrap()[0].clone();
        object = Some(db_object.clone());

        if is_object_borrowed(db_object.id, owner.database_pool.get_conn().unwrap()).0 {
            object = None;
            Text::new("Esine on jo lainattu")
        } else {
            Text::new(format!("Lainattu esine: {:?}", db_object.name))
        }
    } else {
        Text::new("Skannaa objekti")
    };

    let time = chrono::Utc::now();

    if object.is_some() {
        conn.exec_drop(
            r"INSERT INTO itemstorage.borrow_history
        (student_id, object_id, borrow_start_timestamp, borrow_end_timestamp)
        VALUES(:student_id, :object_id, :borrow_start_timestamp, NULL);
        ",
            params! {
                "student_id" => student.id,
                "object_id" => object.unwrap().id,
                "borrow_start_timestamp" => time.timestamp(),
            },
        )
        .unwrap();
    }

    let content = Column::new()
        .spacing(10)
        .push(Text::new("Oppilas"))
        .push(Text::new(format!(
            "{} {}",
            student.first_name, student.last_name
        )))
        .push(Space::with_height(Length::FillPortion(20)))
        .push(message.size(32))
        .push(Space::with_height(Length::FillPortion(25)))
        .align_items(Alignment::Center);

    content
}
