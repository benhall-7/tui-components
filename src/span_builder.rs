use tui::{
    style::Style,
    text::{Span, Spans},
};

#[derive(Debug, Clone, Default)]
pub struct SpanBuilder(Vec<(String, Style)>);

impl SpanBuilder {
    pub fn push(&mut self, string: String, style: Style) {
        self.0.push((string, style));
    }

    pub fn get_spans<'a>(mut self) -> Spans<'a> {
        Spans::from(
            self.0
                .drain(..)
                .map(|(string, style)| Span::styled(string, style))
                .collect::<Vec<_>>(),
        )
    }
}
