use time::OffsetDateTime;

use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind
};

use ratatui::{
    DefaultTerminal,
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Widget, Padding, calendar::{CalendarEventStore, Monthly}}
};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let date = OffsetDateTime::now_utc().date();
        Monthly::new(
            date,
            CalendarEventStore::today(Style::default().red().bold()),
        )
            .block(Block::new().padding(Padding::new(0,0,2,0)))
            .show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC)
            .render(area, buf);

    }
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
