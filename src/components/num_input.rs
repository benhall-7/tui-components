use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent};
use num::traits::{FromPrimitive, SaturatingAdd, SaturatingMul, SaturatingSub};
use num::{Bounded, Float, Integer, Signed, Unsigned};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};
use tui::{buffer::Buffer, layout::Rect};

use crate::{Component, Event};

#[derive(Debug, Clone, Copy)]
pub enum NumInputResponse {
    None,
    Submit,
    Cancel,
}

#[derive(Debug)]
pub struct SignedIntInput<T: InputSignedInt> {
    current: T,
    negative: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NumInputSpanBuilder(Vec<(String, Style)>);

impl<T: InputSignedInt> SignedIntInput<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            current: initial_value,
            negative: initial_value.is_negative(),
        }
    }

    pub fn set(&mut self, value: T) {
        self.current = value.clamp(T::min_value(), T::max_value());
        // If the user removes all digits, keep the sign the same
        if value != T::zero() {
            self.negative = value.is_negative();
        }
    }

    pub fn add(&mut self, value: T) -> &mut Self {
        self.set(self.current.saturating_add(&value));
        self
    }

    pub fn sub(&mut self, value: T) -> &mut Self {
        self.set(self.current.saturating_sub(&value));
        self
    }

    pub fn multiply(&mut self, value: T) -> &mut Self {
        self.set(self.current.saturating_mul(&value));
        self
    }

    pub fn invert(&mut self) {
        if self.current == T::zero() {
            self.negative = !self.negative;
        } else {
            self.set(T::zero().saturating_sub(&self.current))
        }
    }

    pub fn remove_digit(&mut self) {
        // integer division with 10
        self.set(self.current / T::from_u32(10).unwrap())
    }

    pub fn value(&self) -> T {
        self.current
    }

    pub fn append_digit(&mut self, digit: char) -> bool {
        if let Some(dig) = digit.to_digit(10) {
            // instead of converting to string, just multiply by 10 and add/sub
            // the digit. This way, we also cap out at the min/max of the number
            if self.negative {
                self.multiply(T::from_u32(10).unwrap())
                    .sub(T::from_u32(dig).unwrap());
            } else {
                self.multiply(T::from_u32(10).unwrap())
                    .add(T::from_u32(dig).unwrap());
            }
            true
        } else {
            false
        }
    }

    pub fn get_span_builder(&self) -> NumInputSpanBuilder {
        let mut builder = NumInputSpanBuilder::default();
        builder.push(
            String::from(if self.negative { "- " } else { "+ " }),
            Style::default().fg(Color::Green),
        );
        let number_no_sign = if self.current.is_negative() {
            let base = format!("{}", self.current);
            if !base.is_empty() {
                String::from(&format!("{}", self.current)[1..])
            } else {
                base
            }
        } else {
            format!("{}", self.current)
        };
        builder.push(number_no_sign, Style::default());
        if self.current == T::max_value() {
            builder.push(
                String::from(" (max value)"),
                Style::default().fg(Color::Gray),
            )
        } else if self.current == T::min_value() {
            builder.push(
                String::from(" (min value)"),
                Style::default().fg(Color::Gray),
            )
        }
        builder
    }
}

impl<T: InputSignedInt> Component for SignedIntInput<T> {
    type Response = NumInputResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: crate::Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    if !self.append_digit(c) && c == '-' {
                        self.invert();
                    }
                }
                KeyCode::Backspace => {
                    self.remove_digit();
                }
                KeyCode::Up => {
                    self.add(T::one());
                }
                KeyCode::Down => {
                    self.sub(T::one());
                }
                KeyCode::Enter => return NumInputResponse::Submit,
                KeyCode::Esc => return NumInputResponse::Cancel,
                _ => {}
            }
        }
        NumInputResponse::None
    }

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) -> Self::DrawResponse {
        let span_builder = self.get_span_builder();
        let text = Paragraph::new(span_builder.get_spans());
        Widget::render(text, rect, buffer);
    }
}

impl NumInputSpanBuilder {
    fn push(&mut self, string: String, style: Style) {
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

pub trait InputSignedInt:
    Integer
    + Signed
    + Bounded
    + SaturatingAdd
    + SaturatingMul
    + SaturatingSub
    + FromPrimitive
    + Copy
    + Display
{
}
impl<T> InputSignedInt for T where
    T: Integer
        + Signed
        + Bounded
        + SaturatingAdd
        + SaturatingMul
        + SaturatingSub
        + FromPrimitive
        + Copy
        + Display
{
}
