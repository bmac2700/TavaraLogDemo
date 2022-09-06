use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_native::{layout, renderer, Layout, Widget};

pub struct TableSpacer {
    height: f32,
    color: Color,
}

impl TableSpacer {
    pub fn new(height: f32, color: Color) -> Self {
        Self { height, color }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for TableSpacer
where
    Renderer: renderer::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(limits.max().width, self.height))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_radius: self.height,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            self.color,
        );
    }
}

impl<'a, Message> Into<Element<'a, Message>> for TableSpacer {
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}
