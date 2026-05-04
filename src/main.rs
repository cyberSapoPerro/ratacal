use time::OffsetDateTime;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;

use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Widget;
use ratatui::widgets::Padding;
use ratatui::widgets::calendar::CalendarEventStore;
use ratatui::widgets::calendar::Monthly;

use ratatui::layout::Layout;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;

use ratatui::widgets::Paragraph;

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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
            ])
            .split(area);

        Monthly::new(
            OffsetDateTime::now_utc().date(),
            CalendarEventStore::today(Style::default().red().bold()),
        )
            .block(Block::new().padding(Padding::new(0,0,2,0)))
            .show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC)
            .render(chunks[0], buf);

        Paragraph::new("Hello from ratatui!\nPress 'q' to quit.")
            .block(Block::default().title("Info").borders(ratatui::widgets::Borders::ALL))
            .render(chunks[1], buf);

    }
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
