use crossterm::event::KeyCode;
use tui::{buffer::Buffer, layout::Rect, widgets::{Paragraph, Widget}, text::{Spans, Span}, style::{Style, Color}};

use crate::{Component, Event, span_builder::SpanBuilder};

pub const TRUE_CHAR: char = '☑';
pub const FALSE_CHAR: char = '☒';

#[derive(Debug, Default)]
pub struct Checkbox {
    pub value: bool,
}

impl Checkbox {
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    pub fn invert(&mut self) {
        self.value = !self.value;
    }

    pub fn get_span_builder(&self) -> SpanBuilder {
        let mut spans = SpanBuilder::default();
        spans.push("> ".into(), Style::default());
        if self.value {
            spans.push(TRUE_CHAR.to_string(), Style::default().fg(Color::Green));
        } else {
            spans.push(FALSE_CHAR.to_string(), Style::default().fg(Color::Yellow));
        }
        spans
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxResponse {
    Edited,
    None,
    Submit,
    Exit,
}

impl Component for Checkbox {
    type Response = CheckboxResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: crate::Event) -> Self::Response {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('t') | KeyCode::Char('y') => {
                    self.value = true;
                    CheckboxResponse::Edited
                }
                KeyCode::Char('f') | KeyCode::Char('n') => {
                    self.value = false;
                    CheckboxResponse::Edited
                }
                KeyCode::Down | KeyCode::Up => {
                    self.value = !self.value;
                    CheckboxResponse::Edited
                }
                KeyCode::Backspace => CheckboxResponse::Exit,
                KeyCode::Enter => CheckboxResponse::Submit,
                _ => CheckboxResponse::None,
            }
        } else {
            CheckboxResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) -> Self::DrawResponse {
        let spans = Spans::from(vec![
            Span::styled("> ", Style::default()),
            if self.value {
                Span::styled(TRUE_CHAR.to_string(), Style::default().fg(Color::Green))
            } else {
                Span::styled(FALSE_CHAR.to_string(), Style::default().fg(Color::Red))
            }
        ]);
        let paragraph = Paragraph::new(spans);
        Widget::render(paragraph, rect, buffer);
    }
}