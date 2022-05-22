pub mod components;
pub mod rect_ext;
pub mod span_builder;

use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{poll, read, Event as TermEvent, KeyEvent, MouseEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, SetTitle};
use crossterm::ErrorKind;
use tui::backend::CrosstermBackend;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Spans;
use tui::widgets::Widget;
use tui::Terminal;

pub use crossterm;
pub use tui;

pub struct Wrapper<'a, A: App>(pub &'a mut A);

impl<'a, A: App> Widget for Wrapper<'a, A> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.0.draw(area, buf)
    }
}

/// A trait enabling a nested layout of UI components
pub trait Component {
    type Response;
    type DrawResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response;

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) -> Self::DrawResponse;
}

// A trait representing a top-level component
pub trait App {
    fn handle_event(&mut self, event: Event) -> AppResponse;

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer);
}

/// A trait for components that can be rendered as spans
pub trait Spannable {
    fn get_spans<'a, 'b>(&'a self) -> Spans<'b>;
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

pub enum AppResponse {
    Exit,
    None,
}

pub fn run<A: App>(app: &mut A, title: Option<String>) -> Result<(), ErrorKind> {
    let mut should_refresh = true;

    let mut t = setup_terminal(title)?;

    loop {
        if should_refresh {
            t.draw(|f| {
                let size = f.size();
                f.render_widget(Wrapper(app), size);
            })
            .unwrap();
            should_refresh = false;
        }

        if poll(Duration::from_secs_f64(1.0 / 60.0)).unwrap() {
            should_refresh = true;
            let event = read().unwrap();
            let comp_event = match event {
                TermEvent::Resize(..) => continue,
                TermEvent::Mouse(m) => Event::Mouse(m),
                TermEvent::Key(k) => Event::Key(k),
            };
            match app.handle_event(comp_event) {
                AppResponse::Exit => break,
                AppResponse::None => {}
            }
        }
    }

    close_terminal(&mut t)?;
    Ok(())
}

fn setup_terminal(title: Option<String>) -> Result<Terminal<CrosstermBackend<Stdout>>, ErrorKind> {
    if let Some(title) = title {
        execute!(stdout(), SetTitle(&title))?;
    }

    enable_raw_mode()?;
    let mut t = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    t.clear().unwrap();
    Ok(t)
}

fn close_terminal(_t: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), ErrorKind> {
    disable_raw_mode()?;
    Ok(())
}
