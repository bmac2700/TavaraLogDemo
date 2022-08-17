use std::io::ErrorKind;

use iced::Subscription;

#[derive(Debug, PartialEq, Clone)]
pub struct TagInfo {
    pub uid_length: u8,
    pub uid: [u8; 8],
}

fn scan_tag(port_name: String) -> Result<TagInfo, String> {
    let mut serial_buffer = [0u8; 16];
    let mut port = match serialport::new(port_name, 115200)
        .timeout(std::time::Duration::from_millis(250))
        .open()
    {
        Ok(x) => x,
        Err(_) => return Err("Port not found".to_string()),
    };

    loop {
        match port.read(&mut serial_buffer) {
            Ok(_x) => {}
            Err(x) => {
                if x.kind() == ErrorKind::TimedOut {
                    continue;
                }

                return Err("Port lost".to_string());
            }
        };

        if serial_buffer[15] == 0 {
            let mut uid = [0u8; 8];
            for i in 0..7 {
                uid[i] = serial_buffer[1 + i];
            }

            let tag_info = TagInfo {
                uid_length: serial_buffer[0],
                uid,
            };

            return Ok(tag_info);
        } else {
            continue;
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScanEvent {
    SendChannel(std::sync::mpsc::Sender<ScanEvent>),
    UpdatePort(String),
    PortLost,
    TagScanned(TagInfo),
}

enum ScanState {
    DoingNothing,
    DoScanning(std::sync::mpsc::Receiver<ScanEvent>, Option<String>),
}

pub fn scan() -> Subscription<ScanEvent> {
    struct Scan;

    iced_native::subscription::unfold(
        std::any::TypeId::of::<Scan>(),
        ScanState::DoingNothing,
        |state| async move {
            match state {
                ScanState::DoingNothing => {
                    let (sender, receiver) = std::sync::mpsc::channel();

                    (
                        Some(ScanEvent::SendChannel(sender)),
                        ScanState::DoScanning(receiver, None),
                    )
                }
                ScanState::DoScanning(receiver, mut port_name) => {
                    let timeout = std::time::Duration::from_millis(500);

                    if let Ok(event) = receiver.recv_timeout(timeout) {
                        if let ScanEvent::UpdatePort(new_port) = event {
                            return (None, ScanState::DoScanning(receiver, Some(new_port)));
                        }
                    }

                    if let Some(port) = port_name.clone() {
                        println!("Scanning {}!", port);
                        match scan_tag(port) {
                            Ok(x) => {
                                return (
                                    Some(ScanEvent::TagScanned(x)),
                                    ScanState::DoScanning(receiver, port_name),
                                );
                            }
                            Err(_) => {
                                port_name = None;
                                return (
                                    Some(ScanEvent::PortLost),
                                    ScanState::DoScanning(receiver, port_name),
                                );
                            }
                        }
                    }

                    (None, ScanState::DoScanning(receiver, port_name))
                }
            }
        },
    )
}
