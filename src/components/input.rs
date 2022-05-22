use crate::{Component, Event, Spannable};
use crossterm::event::KeyCode;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};

#[derive(Debug, Default, Clone)]
// todo: add cursor
pub struct Input {
    pub value: String,
    pub error: Option<String>,
    pub focused: bool,
    text_style: Style,
    editing_style: Style,
    error_style: Style,
}

impl Input {
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_style = style;
        self
    }

    pub fn editing_style(mut self, style: Style) -> Self {
        self.editing_style = style;
        self
    }

    pub fn error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }
}

#[derive(Debug)]
pub enum InputResponse {
    None,
    Edited { deletion: bool },
    Submit,
    Cancel,
}

impl Component for Input {
    type Response = InputResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.value.push(c);
                    InputResponse::Edited { deletion: false }
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    InputResponse::Edited { deletion: true }
                }
                KeyCode::Enter => InputResponse::Submit,
                KeyCode::Esc => InputResponse::Cancel,
                _ => InputResponse::None,
            }
        } else {
            InputResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let p = Paragraph::new(self.get_spans());
        p.render(rect, buf);
    }
}

impl Spannable for Input {
    fn get_spans<'a, 'b>(&'a self) -> Spans<'b> {
        let mut spans = Spans::default();
        if self.focused {
            spans.0.push(Span::raw("> "));
            spans
                .0
                .push(Span::styled(self.value.clone(), self.editing_style));
            if let Some(e) = &self.error {
                spans
                    .0
                    .push(Span::styled(format!(" {}", e), self.error_style));
            }
        } else {
            let style = if self.error.is_some() {
                self.error_style
            } else {
                self.text_style
            };
            spans.0.push(Span::styled(self.value.clone(), style));
        }
        spans
    }
}
