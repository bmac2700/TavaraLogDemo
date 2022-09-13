//=============================================================================//
//
// Tarkoitus: Sisältää suurimman osan koko ohjelmiston logiikasta.
//
//
//=============================================================================//

use std::time::Instant;

use crate::widgets::TablePane;
use iced::{
    button, executor, pane_grid, pick_list, scrollable, text_input, Application, Command,
    Container, Element, Length, Settings,
};
use mysql::prelude::*;
use mysql::*;

use crate::{
    configuration_file::{read_configuration, write_configuration},
    scanner::{ScanEvent, TagInfo},
    views::{
        main_view::{get_object_info, get_student_info},
        tagfound_view::{Object, Student},
    },
};

pub fn launch() -> iced::Result {
    MainView::run(Settings::default())
}

#[derive(Clone, PartialEq, Eq)]
pub enum MenuState {
    Main,
    Settings,
    TagFound,
    ObjectReturn,
    ObjectNotBorrowed,
    CannotReturnObject,
    AddStudent,
    AddObject,
    RemoveStudent,
    RemoveObject,
    BorrowHistory,
}

#[derive(Debug, Clone)]
pub struct BorrowHistoryObject {
    pub id: i64,
    pub student_id: i64,
    pub object_id: i64,
    pub borrow_start_timestamp: i64,
    pub borrow_end_timestamp: Option<i64>,
}

//Returns (is_borrowed, object_id, borrow_start_timestamp)
pub fn is_object_borrowed(id: i64, mut conn: PooledConn) -> (bool, Option<i64>, Option<i64>) {
    let objects = conn
        .query_map(
            format!(
                r"SELECT * FROM borrow_history where object_id={} and borrow_end_timestamp is null and student_id is not null and object_id is not null LIMIT 1;",
                id
            ),
            |(id, student_id, object_id, borrow_start_timestamp, borrow_end_timestamp)| BorrowHistoryObject {
                id,
                student_id,
                object_id,
                borrow_start_timestamp,
                borrow_end_timestamp,
            },
        )
        .unwrap();

    if objects.len() == 1 {
        (
            true,
            Some(objects[0].id),
            Some(objects[0].borrow_start_timestamp),
        )
    } else {
        (false, None, None)
    }
}

pub struct MainView {
    pub menu_state: MenuState,

    pub initialized: bool,

    pub database_pool: Pool,

    pub scanner_channel: Option<std::sync::mpsc::Sender<ScanEvent>>,

    pub last_scanned_student_tag: Option<TagInfo>,
    pub last_scanned_object_tag: Option<TagInfo>,
    pub new_tag: Option<TagInfo>,
    pub teacher_tag: Option<TagInfo>,

    pub selected_device: Option<String>,

    pub first_name_input: text_input::State,
    pub first_name_value: String,

    pub last_name_input: text_input::State,
    pub last_name_value: String,

    pub group_tag_input: text_input::State,
    pub group_tag_value: String,

    pub part_number_input: text_input::State,
    pub part_number_value: String,

    pub manufacturer_input: text_input::State,
    pub manufacturer_value: String,

    pub location_input: text_input::State,
    pub location_value: String,

    pub student_id_input: text_input::State,
    pub student_id_value: String,

    pub object_id_input: text_input::State,
    pub object_id_value: String,

    pub object_name_input: text_input::State,
    pub object_name_value: String,

    pub student_search_input: text_input::State,
    pub student_search_value: String,

    pub object_search_input: text_input::State,
    pub object_search_value: String,

    //Pick Lists
    pub device_list: pick_list::State<String>,

    //Buttons
    pub settings_button: button::State,
    pub history_button: button::State,
    pub add_student_button: button::State,
    pub add_student_view: button::State,
    pub make_new_student_admin: bool,

    pub back_to_mainscreen: button::State,
    pub add_object_button: button::State,
    pub add_object_view: button::State,

    pub remove_student_view: button::State,
    pub remove_student_button: button::State,

    pub remove_object_view: button::State,
    pub remove_object_button: button::State,

    pub borrow_list: scrollable::State,
    pub student_list: scrollable::State,
    pub object_list: scrollable::State,

    pub borrow_history: scrollable::State,

    pub object_list_panes: pane_grid::State<TablePane>,
    pub borrow_list_panes: pane_grid::State<TablePane>,
    pub borrow_history_panes: pane_grid::State<TablePane>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SettingsButtonClick,
    DeviceSelected(String),
    ScanEventRecv(crate::scanner::ScanEvent),
    Tick(Instant),
    FirstNameChanged(String),
    LastNameChanged(String),
    ObjectNameChanged(String),
    StudentIdChanged(String),
    ObjectIdChanged(String),
    GroupTagChanged(String),
    PartNumberChanged(String),
    ManufacturerChanged(String),
    LocationChanged(String),
    ObjectFilterChanged(String),
    AddStudentViewButton,
    AddStudentButton,
    BackToSettings,
    AddObjectViewButton,
    AddObjectButton,
    RemoveStudentViewButton,
    RemoveStudentButton,
    RemoveObjectViewButton,
    RemoveObjectButton,
    HistoryButtonClick,
    StudentSearchChanged(String),
    ToggleAdmin(bool),
}

impl Application for MainView {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let url: &str = &read_configuration("./config.toml").unwrap().database_url;
        let pool = Pool::new(url).unwrap();

        {
            let mut conn = pool.get_conn().unwrap();

            conn.exec_drop(
                r"CREATE TABLE IF NOT EXISTS students (
                id BIGINT auto_increment NULL,
                first_name TEXT NULL,
                last_name TEXT NULL,
                group_tag TEXT NULL,
                uid_length INT NULL,
                uid BIGINT NULL,
                admin tinyint(1) DEFAULT NULL,
                CONSTRAINT students_pk PRIMARY KEY (id)
            )
            ENGINE=InnoDB
            DEFAULT CHARSET=latin1
            COLLATE=latin1_swedish_ci;",
                (),
            )
            .unwrap();

            conn.exec_drop(
                r"CREATE TABLE IF NOT EXISTS objects (
                id BIGINT auto_increment NULL,
                name TEXT NULL,
                part_number TEXT NULL,
                manufacturer TEXT NULL,
                location TEXT NULL,
                uid_length INT NULL,
                uid BIGINT NULL,
                CONSTRAINT students_pk PRIMARY KEY (id)
            )
            ENGINE=InnoDB
            DEFAULT CHARSET=latin1
            COLLATE=latin1_swedish_ci;",
                (),
            )
            .unwrap();

            conn.exec_drop(r"CREATE TABLE IF NOT EXISTS borrow_history (
                id BIGINT auto_increment NULL,
                student_id BIGINT NULL,
                object_id BIGINT NULL,
                borrow_start_timestamp BIGINT NULL,
                borrow_end_timestamp BIGINT NULL,
                CONSTRAINT borrow_history_pk PRIMARY KEY (id),
                CONSTRAINT borrow_history_FK FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE SET NULL,
                CONSTRAINT borrow_history_FK_1 FOREIGN KEY (object_id) REFERENCES objects(id) ON DELETE SET NULL
            )
            ENGINE=InnoDB
            DEFAULT CHARSET=latin1
            COLLATE=latin1_swedish_ci;
            ", ()).unwrap();
        }

        let object_list_panes = {
            let (mut panes, pane) = pane_grid::State::new(TablePane::new(0));
            let second_pane = panes
                .split(pane_grid::Axis::Vertical, &pane, TablePane { id: 1 })
                .unwrap();
            panes.split(pane_grid::Axis::Vertical, &pane, TablePane { id: 2 });
            panes.split(
                pane_grid::Axis::Vertical,
                &second_pane.0,
                TablePane { id: 3 },
            );

            panes
        };

        let borrow_list_panes = {
            let (mut panes, pane) = pane_grid::State::new(TablePane::new(0));
            let x = panes
                .split(pane_grid::Axis::Vertical, &pane, TablePane { id: 2 })
                .unwrap();
            panes.split(pane_grid::Axis::Vertical, &pane, TablePane { id: 1 });

            panes.resize(&x.1, 0.65);
            panes
        };

        let borrow_history_panes = {
            let (mut panes, pane) = pane_grid::State::new(TablePane::new(0));
            let second_pane = panes
                .split(pane_grid::Axis::Vertical, &pane, TablePane { id: 1 })
                .unwrap();
            panes.split(pane_grid::Axis::Vertical, &pane, TablePane { id: 2 });
            panes.split(
                pane_grid::Axis::Vertical,
                &second_pane.0,
                TablePane { id: 3 },
            );

            panes
        };

        (
            MainView {
                initialized: false,
                menu_state: MenuState::Main,
                database_pool: pool,
                scanner_channel: None,
                last_scanned_student_tag: None,
                last_scanned_object_tag: None,
                new_tag: None,
                selected_device: None,
                first_name_input: text_input::State::default(),
                first_name_value: String::default(),
                last_name_input: text_input::State::default(),
                last_name_value: String::default(),
                object_name_input: text_input::State::default(),
                object_name_value: String::default(),
                student_id_input: text_input::State::default(),
                student_id_value: String::default(),
                device_list: pick_list::State::default(),
                settings_button: button::State::default(),
                add_student_button: button::State::default(),
                add_student_view: button::State::default(),
                back_to_mainscreen: button::State::default(),
                add_object_button: button::State::default(),
                add_object_view: button::State::default(),
                borrow_list: scrollable::State::default(),
                student_list: scrollable::State::default(),
                remove_student_view: button::State::default(),
                remove_student_button: button::State::default(),
                object_id_input: text_input::State::default(),
                object_id_value: String::default(),
                remove_object_button: button::State::default(),
                remove_object_view: button::State::default(),
                object_list: scrollable::State::default(),
                history_button: button::State::default(),
                borrow_history: scrollable::State::default(),
                teacher_tag: None,
                student_search_input: text_input::State::default(),
                student_search_value: String::default(),
                make_new_student_admin: false,
                group_tag_input: text_input::State::default(),
                group_tag_value: String::default(),
                manufacturer_input: text_input::State::default(),
                manufacturer_value: String::default(),
                part_number_input: text_input::State::default(),
                part_number_value: String::default(),
                location_input: text_input::State::default(),
                location_value: String::default(),
                object_list_panes,
                borrow_list_panes,
                borrow_history_panes,
                object_search_input: text_input::State::default(),
                object_search_value: String::default(),
            },
            Command::none(),
        )
    }

    fn scale_factor(&self) -> f64 {
        if self.menu_state == MenuState::Main
            || self.menu_state == MenuState::ObjectReturn
            || self.menu_state == MenuState::CannotReturnObject
            || self.menu_state == MenuState::TagFound
            || self.menu_state == MenuState::ObjectNotBorrowed
        {
            return 2f64;
        }

        return 1f64;
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let mut subscriptions = vec![crate::scanner::scan().map(Message::ScanEventRecv)];

        if self.menu_state == MenuState::TagFound {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_secs(5)).map(Message::Tick));
        }

        if self.menu_state == MenuState::ObjectReturn {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_secs(2)).map(Message::Tick));
        }

        if self.menu_state == MenuState::ObjectNotBorrowed {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_secs(1)).map(Message::Tick));
        }

        if self.menu_state == MenuState::CannotReturnObject {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_secs(3)).map(Message::Tick));
        }

        if self.menu_state == MenuState::Main || self.menu_state == MenuState::Settings {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_millis(500)).map(Message::Tick));
        }

        iced::Subscription::batch(subscriptions)
    }

    fn mode(&self) -> iced::window::Mode {
        #[cfg(debug_assertions)]
        return iced::window::Mode::Windowed;

        #[cfg(not(debug_assertions))]
        return iced::window::Mode::Fullscreen;
    }

    fn title(&self) -> String {
        String::from("Tavara Log Demo")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if self.selected_device.is_none() && !self.initialized && self.scanner_channel.is_some() {
            let config = read_configuration("./config.toml").unwrap();

            if !config.scanner_port.is_empty() {
                let x = self.scanner_channel.as_ref().unwrap();
                x.send(ScanEvent::UpdatePort(config.scanner_port)).unwrap();
                self.selected_device = Some(config.scanner_port_name);
            }

            self.initialized = true;
        }
        match message {
            Message::ObjectFilterChanged(val) => {
                self.object_search_value = val;
            }
            Message::LocationChanged(val) => {
                self.location_value = val;
            }
            Message::PartNumberChanged(val) => {
                self.part_number_value = val;
            }
            Message::ManufacturerChanged(val) => {
                self.manufacturer_value = val;
            }
            Message::GroupTagChanged(val) => {
                self.group_tag_value = val;
            }
            Message::ToggleAdmin(val) => {
                self.make_new_student_admin = val;
            }
            Message::StudentSearchChanged(x) => {
                self.student_search_value = x;
            }
            Message::HistoryButtonClick => {
                self.menu_state = MenuState::BorrowHistory;
                self.student_search_value = "".to_string();
                self.object_search_value = "".to_string();
            }
            Message::RemoveObjectButton => {
                let mut conn = self.database_pool.get_conn().unwrap();

                let id: i64 = self.object_id_value.parse().unwrap();

                let object_info = get_object_info(id, &mut conn);

                if object_info.is_some() {
                    conn.exec_drop(r"DELETE FROM objects where id=:id", params! {"id" => id})
                        .unwrap();
                    println!("Removed object {}", id);

                    self.menu_state = MenuState::Settings;
                    self.object_id_value = "".to_string();
                }
            }
            Message::RemoveObjectViewButton => {
                self.menu_state = MenuState::RemoveObject;
            }
            Message::ObjectIdChanged(x) => {
                let mut real_text = String::new();

                for c in x.chars() {
                    if !c.is_numeric() {
                        continue;
                    }

                    real_text.push(c);
                }

                self.object_id_value = real_text;
            }
            Message::RemoveStudentButton => {
                let mut conn = self.database_pool.get_conn().unwrap();

                let id: i64 = self.student_id_value.parse().unwrap();

                let student_info = get_student_info(id, &mut conn);

                if student_info.is_some() {
                    conn.exec_drop(r"DELETE FROM students where id=:id", params! {"id" => id})
                        .unwrap();
                    println!("Removed student {}", id);

                    self.menu_state = MenuState::Settings;
                    self.student_id_value = "".to_string();
                }
            }
            Message::RemoveStudentViewButton => {
                self.menu_state = MenuState::RemoveStudent;
            }
            Message::StudentIdChanged(x) => {
                let mut real_text = String::new();

                for c in x.chars() {
                    if !c.is_numeric() {
                        continue;
                    }

                    real_text.push(c);
                }

                self.student_id_value = real_text;
            }
            Message::SettingsButtonClick => {
                self.student_search_value = "".to_string();
                if self.menu_state != MenuState::Settings {
                    self.menu_state = MenuState::Settings;
                } else {
                    self.menu_state = MenuState::Main;
                    self.teacher_tag = None;
                    self.new_tag = None;
                }
            }
            Message::AddObjectViewButton => {
                self.menu_state = MenuState::AddObject;
            }
            Message::AddStudentViewButton => {
                self.menu_state = MenuState::AddStudent;
            }
            Message::BackToSettings => {
                self.first_name_value = "".to_string();
                self.last_name_value = "".to_string();
                self.object_name_value = "".to_string();
                self.student_search_value = "".to_string();
                self.part_number_value = "".to_string();
                self.manufacturer_value = "".to_string();
                self.location_value = "".to_string();
                self.group_tag_value = "".to_string();
                self.new_tag = None;
                self.make_new_student_admin = false;
                self.menu_state = MenuState::Settings;
            }
            Message::AddObjectButton => {
                if self.new_tag.is_some() && !self.object_name_value.is_empty() {
                    let tag = self.new_tag.clone().unwrap();
                    let tag_hash = i64::from_be_bytes(tag.uid);

                    println!("{} | {}", self.object_name_value, tag_hash);
                    let mut conn = self.database_pool.get_conn().unwrap();
                    conn.exec_drop(
                        "INSERT INTO objects
                        (name, part_number, manufacturer, location, uid_length, uid)
                        VALUES(:name, :part_number, :manufacturer, :location, :uid_length, :uid);
                        ",
                        params! {
                            "name" => self.object_name_value.clone(),
                            "part_number" => self.part_number_value.clone(),
                            "manufacturer" => self.manufacturer_value.clone(),
                            "location" => self.location_value.clone(),
                            "uid_length" => tag.uid_length,
                            "uid" => tag_hash,
                        },
                    )
                    .unwrap();

                    self.menu_state = MenuState::Settings;

                    self.first_name_value = "".to_string();
                    self.last_name_value = "".to_string();
                    self.object_name_value = "".to_string();
                    self.part_number_value = "".to_string();
                    self.manufacturer_value = "".to_string();
                    self.location_value = "".to_string();
                    self.new_tag = None;
                }
            }
            Message::ObjectNameChanged(x) => {
                self.object_name_value = x;
            }
            Message::AddStudentButton => {
                if self.new_tag.is_some()
                    && !self.first_name_value.is_empty()
                    && !self.last_name_value.is_empty()
                {
                    let tag = self.new_tag.clone().unwrap();
                    let tag_hash = i64::from_be_bytes(tag.uid);

                    println!(
                        "{} {} | {}",
                        self.first_name_value,
                        self.last_name_value,
                        i64::from_be_bytes(tag.uid)
                    );
                    let mut conn = self.database_pool.get_conn().unwrap();
                    conn.exec_drop(
                        "INSERT INTO students
                    (first_name, last_name, group_tag, uid_length, uid, admin)
                    VALUES(:first_name, :last_name, :group_tag, :uid_length, :uid, :admin);
                    ",
                        params! {
                            "first_name" => self.first_name_value.clone(),
                            "last_name" => self.last_name_value.clone(),
                            "group_tag" => self.group_tag_value.clone(),
                            "uid_length" => tag.uid_length,
                            "uid" => tag_hash,
                            "admin" => self.make_new_student_admin,
                        },
                    )
                    .unwrap();

                    self.menu_state = MenuState::Settings;

                    self.first_name_value = "".to_string();
                    self.last_name_value = "".to_string();
                    self.object_name_value = "".to_string();
                    self.group_tag_value = "".to_string();
                    self.new_tag = None;
                    self.make_new_student_admin = false;
                }
            }
            Message::DeviceSelected(pretty_port_name) => {
                self.selected_device = Some(pretty_port_name.clone());

                //Removes the " - <Device name>" part

                let new_port = pretty_port_name
                    .clone()
                    .split(" - ")
                    .take(1)
                    .collect::<Vec<_>>()[0]
                    .to_string();

                let mut config = read_configuration("./config.toml").unwrap();
                config.scanner_port = new_port.clone();
                config.scanner_port_name = pretty_port_name.clone();

                write_configuration("./config.toml", &config);

                self.scanner_channel
                    .as_mut()
                    .unwrap()
                    .send(ScanEvent::UpdatePort(new_port))
                    .unwrap();
            }
            Message::FirstNameChanged(x) => {
                self.first_name_value = x;
            }
            Message::LastNameChanged(x) => {
                self.last_name_value = x;
            }
            Message::ScanEventRecv(event) => match event {
                ScanEvent::SendChannel(channel) => {
                    self.scanner_channel = Some(channel);
                }
                ScanEvent::TagScanned(tag) => {
                    println!("{:?} {}", tag, i64::from_be_bytes(tag.uid));

                    if self.menu_state == MenuState::TagFound && self.new_tag.is_some() {
                        return Command::none();
                    }

                    if self.menu_state == MenuState::AddStudent
                        || self.menu_state == MenuState::AddObject
                    {
                        self.new_tag = Some(tag);
                        return Command::none();
                    }

                    if self.menu_state == MenuState::Settings {
                        self.teacher_tag = Some(tag);
                        return Command::none();
                    }

                    if self.last_scanned_object_tag.is_none() {
                        let tag = tag.clone();
                        let val = i64::from_be_bytes(tag.uid);

                        let mut conn = self.database_pool.get_conn().unwrap();

                        //Ei ole hyvä tapa, mahdollinen sql injection, mutta todella epätodennäköinen sillä tagin uid pitäisi olla tekstiä, mutta jos se olisi aika varmasti arduino luulee sitä vialliseksi eikä lähetä sitä
                        let objects = conn
                            .query_map(
                                format!(r"SELECT * FROM objects where uid={} LIMIT 1;", val),
                                |(
                                    id,
                                    name,
                                    part_number,
                                    manufacturer,
                                    location,
                                    uid_length,
                                    uid,
                                )| Object {
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
                            if self.last_scanned_student_tag.is_some() {
                                self.last_scanned_object_tag = Some(tag);
                                self.menu_state = MenuState::TagFound;
                            } else {
                                let is_borrowed = is_object_borrowed(
                                    objects[0].id,
                                    self.database_pool.get_conn().unwrap(),
                                );
                                if is_borrowed.0 {
                                    if is_borrowed.2.unwrap() + 10 <= chrono::Utc::now().timestamp()
                                    {
                                        self.menu_state = MenuState::ObjectReturn;
                                        conn.exec_drop(r"UPDATE borrow_history SET borrow_end_timestamp=:borrow_end_timestamp WHERE id=:id;", params!{
                                        "id" => is_borrowed.1.unwrap(),
                                        "borrow_end_timestamp" => chrono::Utc::now().timestamp(),
                                    }).unwrap()
                                    } else {
                                        self.menu_state = MenuState::CannotReturnObject;
                                    }
                                } else {
                                    self.menu_state = MenuState::ObjectNotBorrowed;
                                }
                            }
                            println!("Object scanned: {:?}", objects[0]);
                        }
                    }

                    if self.last_scanned_student_tag.is_none() {
                        let val = i64::from_be_bytes(tag.uid);
                        let mut conn = self.database_pool.get_conn().unwrap();

                        //Ei ole hyvä tapa, mahdollinen sql injection, mutta todella epätodennäköinen sillä tagin uid pitäisi olla tekstiä, mutta jos se olisi aika varmasti arduino luulee sitä vialliseksi eikä lähetä sitä
                        let students = conn
                            .query_map(
                                format!(r"SELECT * FROM students where uid={} LIMIT 1;", val),
                                |(id, first_name, last_name, group_tag, uid_length, uid, admin)| {
                                    Student {
                                        id,
                                        first_name,
                                        last_name,
                                        group_tag,
                                        uid_length,
                                        uid,
                                        admin,
                                    }
                                },
                            )
                            .unwrap();

                        if students.len() == 1 {
                            self.last_scanned_student_tag = Some(tag);
                            println!("Student scanned: {:?}", students[0]);
                            self.menu_state = MenuState::TagFound;
                        }
                        return Command::none();
                    }
                }
                ScanEvent::PortLost => {
                    self.selected_device = None;
                }
                _ => {}
            },
            Message::Tick(_time) => {
                if self.menu_state == MenuState::TagFound {
                    self.menu_state = MenuState::Main;
                    self.last_scanned_student_tag = None;
                    self.last_scanned_object_tag = None;
                }

                if self.menu_state == MenuState::ObjectReturn
                    || self.menu_state == MenuState::ObjectNotBorrowed
                    || self.menu_state == MenuState::CannotReturnObject
                {
                    self.menu_state = MenuState::Main;
                }

                if self.menu_state == MenuState::Settings {}
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content = match self.menu_state {
            MenuState::Main => crate::views::main_view::get_view(self),
            MenuState::Settings => crate::views::settings_view::get_view(self),
            MenuState::TagFound => crate::views::tagfound_view::get_view(self),
            MenuState::ObjectReturn => crate::views::object_return_view::get_view(self),
            MenuState::ObjectNotBorrowed => crate::views::object_not_borrowed::get_view(self),
            MenuState::CannotReturnObject => crate::views::cannot_return_object::get_view(self),
            MenuState::AddStudent => crate::views::add_student_view::get_view(self),
            MenuState::AddObject => crate::views::add_object_view::get_view(self),
            MenuState::RemoveStudent => crate::views::remove_student_view::get_view(self),
            MenuState::RemoveObject => crate::views::remove_object_view::get_view(self),
            MenuState::BorrowHistory => crate::views::borrow_history_view::get_view(self),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
