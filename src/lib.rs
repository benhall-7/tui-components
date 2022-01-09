pub mod components;
pub mod rect_ext;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::{Widget};

pub use tui;
pub use crossterm;

pub struct Wrapper<'a, App: Component>(pub &'a mut App);

impl<'a, App: Component> Widget for Wrapper<'a, App> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.0.draw(area, buf)
    }
}

/// A trait enabling a nested layout of structs
pub trait Component {
    type Response;

    fn handle_event(&mut self, event: Event) -> Self::Response;

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer);
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
}
