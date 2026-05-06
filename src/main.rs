use time::OffsetDateTime;
use time::Time;
use time::format_description::parse;
use time::macros::format_description;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event;

use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::Widget;
use ratatui::widgets::calendar::CalendarEventStore;
use ratatui::widgets::calendar::Monthly;

#[derive(Debug)]
struct Entry {
    description: String,
    hour: Time,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    entries: Vec<Entry>,

    // input state
    input_mode: InputMode,
    input_desc: String,
    input_hour: String,
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    EditingDescription,
    EditingHour,
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

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal =>
                match key.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('a') => {
                        self.input_mode = InputMode::EditingDescription;
                    }
                    _ => {}
                },

            InputMode::EditingDescription => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::EditingHour;
                }
                KeyCode::Char(c) => {
                    self.input_desc.push(c);
                }
                KeyCode::Backspace => {
                    self.input_desc.pop();
                }
                KeyCode::Esc => {
                    self.cancel_input();
                }
                _ => {}
            },

            InputMode::EditingHour => match key.code {
                KeyCode::Enter => {
                    self.submit_event();
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.input_hour.push(c);
                }
                KeyCode::Backspace => {
                    self.input_hour.pop();
                }
                KeyCode::Esc => {
                    self.cancel_input();
                }
                _ => {}
            },
        }
    }

    fn submit_event(&mut self) {
        // TODO Handle errors properly
        let format = parse("[hour][minute]").unwrap();
        let hour_str = self.input_hour.clone();
        let hour = Time::parse(&hour_str, &format).unwrap();

        self.entries.push(Entry {
            description: self.input_desc.clone(),
            hour,
        });
        self.entries.sort_by_key(|e| e.hour);
        self.reset_input();
    }

    fn cancel_input(&mut self) {
        self.reset_input();
    }

    fn reset_input(&mut self) {
        self.input_desc.clear();
        self.input_hour.clear();
        self.input_mode = InputMode::Normal;
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
                Constraint::Length(3),
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

        let format = format_description!("[hour]:[minute]");
        let mut rows: Vec<Row> = Vec::new();
        for e in &self.entries {
            let hour_str = e.hour.format(&format).unwrap();
            rows.push(Row::new(vec![
                Cell::from(hour_str),
                Cell::from(e.description.as_str()),
            ]));
        }

        let widths = [
            Constraint::Length(5),
            Constraint::Length(5),
        ];

        Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec!["Hour", "Desc"])
                .style(Style::new().bold())
                .bottom_margin(1),
                )
            .block(Block::new().title("Today").borders(ratatui::widgets::Borders::ALL))
            .highlight_symbol(">>")
            .render(chunks[1], buf);

        let input = match self.input_mode {
            InputMode::EditingDescription => {
                Paragraph::new(format!("Desc: {}", self.input_desc))
            }
            InputMode::EditingHour => {
                Paragraph::new(format!("Hour: {}", self.input_hour))
            }
            InputMode::Normal => {
                Paragraph::new("Press 'a' to add events")
            }
        };

        input.block(Block::default().title("New Event").borders(Borders::ALL))
            .render(chunks[2], buf);
    }
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
