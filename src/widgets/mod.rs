pub mod spacer;

pub struct TablePane {
    pub id: usize,
}

impl TablePane {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

pub mod style {
    use iced::{container, Background, Color};

    const SURFACE: Color = Color::from_rgb(
        0xF2 as f32 / 255.0,
        0xF3 as f32 / 255.0,
        0xF5 as f32 / 255.0,
    );

    pub enum Theme {
        Primary,
    }

    impl container::StyleSheet for Theme {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
                border_width: 1.0,
                border_color: Color::BLACK,
                ..Default::default()
            }
        }
    }
}
