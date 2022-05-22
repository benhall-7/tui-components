use std::fmt::Display;
use std::marker::PhantomData;

use crossterm::event::KeyCode;
use num::traits::{FromPrimitive, SaturatingAdd, SaturatingMul, SaturatingSub};
use num::{Bounded, Float, Integer, Signed, Unsigned};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};
use tui::{buffer::Buffer, layout::Rect};

use crate::span_builder::SpanBuilder;
use crate::{Component, Event, Spannable};

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
        let text = Paragraph::new(self.get_spans());
        Widget::render(text, rect, buffer);
    }
}

impl<T: InputSignedInt> Spannable for SignedIntInput<T> {
    fn get_spans<'a, 'b>(&'a self) -> tui::text::Spans<'b> {
        let mut spans = Spans::default();
        spans.0.push(Span::styled(
            String::from(if self.negative { "- " } else { "+ " }),
            Style::default().fg(Color::Green),
        ));
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
        spans.0.push(Span::raw(number_no_sign));
        if self.current == T::max_value() {
            spans.0.push(Span::styled(
                String::from(" (max value)"),
                Style::default().fg(Color::Gray),
            ))
        } else if self.current == T::min_value() {
            spans.0.push(Span::styled(
                String::from(" (min value)"),
                Style::default().fg(Color::Gray),
            ))
        }
        spans
    }
}

#[derive(Debug)]
pub struct UnsignedIntInput<T: InputUnsignedInt> {
    current: T,
}

impl<T: InputUnsignedInt> UnsignedIntInput<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            current: initial_value,
        }
    }

    pub fn set(&mut self, value: T) {
        self.current = value.clamp(T::min_value(), T::max_value());
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
            self.multiply(T::from_u32(10).unwrap())
                .add(T::from_u32(dig).unwrap());
            true
        } else {
            false
        }
    }
}

impl<T: InputUnsignedInt> Component for UnsignedIntInput<T> {
    type Response = NumInputResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: crate::Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.append_digit(c);
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
        let text = Paragraph::new(self.get_spans());
        Widget::render(text, rect, buffer);
    }
}

impl<T: InputUnsignedInt> Spannable for UnsignedIntInput<T> {
    fn get_spans<'a, 'b>(&'a self) -> Spans<'b> {
        let mut spans = Spans::default();
        spans.0.push(Span::styled(
            String::from("> "),
            Style::default().fg(Color::Green),
        ));
        spans.0.push(Span::raw(format!("{}", self.current)));
        if self.current == T::max_value() {
            spans.0.push(Span::styled(
                String::from(" (max value)"),
                Style::default().fg(Color::Gray),
            ))
        } else if self.current == T::min_value() {
            spans.0.push(Span::styled(
                String::from(" (min value)"),
                Style::default().fg(Color::Gray),
            ))
        }
        spans
    }
}

#[derive(Debug)]
pub struct FloatInput<T: InputFloat> {
    value: FloatValue,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub enum FloatValue {
    Infinity { negative: bool },
    Nan,
    Number(FloatNum),
}

#[derive(Debug)]
pub struct FloatNum {
    whole: String,
    integral: Option<String>,
    negative: bool,
}

#[derive(Debug)]
pub enum NewFloatError {
    /// The float was neither infinity, Nan, or finite
    InvalidState,
    /// The float's string representation isn't a valid decimal
    ParseError,
}

fn parse_digit_string(string: &str) -> Result<&str, NewFloatError> {
    if string.chars().all(|c| char::is_ascii_digit(&c)) {
        Ok(string)
    } else {
        Err(NewFloatError::ParseError)
    }
}

impl<T: InputFloat> FloatInput<T> {
    pub fn new(initial_value: T) -> Result<Self, NewFloatError> {
        let value = if initial_value.is_infinite() {
            FloatValue::Infinity {
                negative: initial_value.is_sign_negative(),
            }
        } else if initial_value.is_nan() {
            FloatValue::Nan
        } else if initial_value.is_finite() {
            let repr = initial_value.to_string();
            // TODO: make number parsing cleaner
            let has_decimal = repr.contains('.');
            let is_negative = repr.chars().next().map_or(false, |c| c == '-');
            let repr_no_sign = if is_negative { &repr[1..] } else { &repr[..] };
            if has_decimal {
                let (first_maybe, second_maybe) = repr_no_sign.split_once('.').unwrap();
                if first_maybe.is_empty() && second_maybe.is_empty() {
                    return Err(NewFloatError::ParseError);
                }
                FloatValue::Number(FloatNum {
                    whole: parse_digit_string(first_maybe)?.into(),
                    integral: Some(parse_digit_string(second_maybe)?.into()),
                    negative: is_negative,
                })
            } else {
                if repr_no_sign.is_empty() {
                    return Err(NewFloatError::ParseError);
                }
                FloatValue::Number(FloatNum {
                    whole: parse_digit_string(repr_no_sign)?.into(),
                    integral: None,
                    negative: is_negative,
                })
            }
        } else {
            return Err(NewFloatError::ParseError);
        };
        Ok(FloatInput {
            value,
            _phantom: PhantomData::default(),
        })
    }

    pub fn push_digit(&mut self, digit: char) {
        if let FloatValue::Number(value) = &mut self.value {
            if digit.is_ascii_digit() {
                value
                    .integral
                    .as_mut()
                    .unwrap_or(&mut value.whole)
                    .push(digit);
                if value.integral.is_none() {
                    value.whole = value.whole.trim_start_matches('0').into();
                }
            } else if digit == '.' && value.integral.is_none() {
                value.integral = Some("".into());
            }
        }
    }

    pub fn remove_digit(&mut self) {
        if let FloatValue::Number(value) = &mut self.value {
            if let Some(integral) = &mut value.integral {
                if integral.is_empty() {
                    value.integral = None;
                } else {
                    integral.pop();
                }
            } else {
                value.whole.pop();
            }
        }
    }

    pub fn value(&self) -> T {
        match &self.value {
            FloatValue::Infinity {
                negative: is_negative,
            } => {
                if *is_negative {
                    T::neg_infinity()
                } else {
                    T::infinity()
                }
            }
            FloatValue::Nan => T::nan(),
            FloatValue::Number(number) => {
                let whole_part = if number.whole.is_empty() {
                    "0"
                } else {
                    &number.whole
                };
                let entire = if let Some(integral) = &number.integral {
                    format!("{}.{}", whole_part, integral)
                } else {
                    whole_part.to_string()
                };

                let raw = T::from_str_radix(&entire, 10)
                    .map_err(|_| NewFloatError::ParseError)
                    .unwrap();
                if number.negative {
                    -raw
                } else {
                    raw
                }
            }
        }
    }
}

impl<T: InputFloat> Component for FloatInput<T> {
    type Response = NumInputResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: crate::Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    if c.is_ascii_digit() || c == '.' {
                        self.push_digit(c)
                    } else if c == '-' {
                        match &mut self.value {
                            FloatValue::Number(num) => num.negative = !num.negative,
                            FloatValue::Infinity {
                                negative: is_negative,
                            } => *is_negative = !*is_negative,
                            _ => {}
                        }
                    }
                }
                KeyCode::Backspace => {
                    self.remove_digit();
                }
                KeyCode::Tab => match &self.value {
                    FloatValue::Number(..) => self.value = FloatValue::Infinity { negative: false },
                    FloatValue::Infinity { .. } => {
                        self.value = FloatValue::Nan;
                    }
                    FloatValue::Nan => {
                        self.value = FloatValue::Number(FloatNum {
                            whole: String::new(),
                            integral: None,
                            negative: false,
                        })
                    }
                },
                KeyCode::Enter => return NumInputResponse::Submit,
                KeyCode::Esc => return NumInputResponse::Cancel,
                _ => {}
            }
        }
        NumInputResponse::None
    }

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) -> Self::DrawResponse {
        let text = Paragraph::new(self.get_spans());
        Widget::render(text, rect, buffer);
    }
}

impl<T: InputFloat> Spannable for FloatInput<T> {
    fn get_spans<'a, 'b>(&'a self) -> Spans<'b> {
        let mut spans = Spans::default();
        match &self.value {
            FloatValue::Infinity { negative } => {
                spans.0.push(Span::styled(
                    String::from(if *negative { "- " } else { "+ " }),
                    Style::default().fg(Color::Green),
                ));
                spans.0.push(Span::raw(T::infinity().to_string()));
            }
            FloatValue::Nan => {
                spans.0.push(Span::styled(
                    String::from("> "),
                    Style::default().fg(Color::Green),
                ));
                spans.0.push(Span::raw(T::nan().to_string()));
            }
            FloatValue::Number(number) => {
                let whole_part = if number.whole.is_empty() {
                    "0"
                } else {
                    &number.whole
                };
                let entire = if let Some(integral) = &number.integral {
                    format!("{}.{}", whole_part, integral)
                } else {
                    whole_part.to_string()
                };
                spans.0.push(Span::styled(
                    String::from(if number.negative { "- " } else { "+ " }),
                    Style::default().fg(Color::Green),
                ));
                spans.0.push(Span::raw(entire));
            }
        }
        spans
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

pub trait InputUnsignedInt:
    Integer
    + Unsigned
    + Bounded
    + SaturatingAdd
    + SaturatingMul
    + SaturatingSub
    + FromPrimitive
    + Copy
    + Display
{
}

impl<T> InputUnsignedInt for T where
    T: Integer
        + Unsigned
        + Bounded
        + SaturatingAdd
        + SaturatingMul
        + SaturatingSub
        + FromPrimitive
        + Copy
        + Display
{
}

pub trait InputFloat: Float + Signed + FromPrimitive + Copy + Display {}

impl<T> InputFloat for T where T: Float + Signed + FromPrimitive + Copy + Display {}
