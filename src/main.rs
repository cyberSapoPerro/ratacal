use time::Date;
use time::Duration;
use time::Month;
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
use ratatui::layout::Alignment;
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
    day: Date,
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    entries: Vec<Entry>,
    date: Date,

    // input state
    input_mode: InputMode,
    input_date: String,
    input_desc: String,
    input_hour: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            entries: Vec::new(),
            date: OffsetDateTime::now_utc().date(),
            input_mode: InputMode::default(),
            input_date: String::new(),
            input_desc: String::new(),
            input_hour: String::new(),
        }
    }
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    EditingDate,
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
                        self.input_mode = InputMode::EditingDate;
                    }
                    KeyCode::Char('t') => {
                        self.date += Duration::days(1);
                    }
                    KeyCode::Char('T') => {
                        self.date -= Duration::days(1);
                    }
                    KeyCode::Char('w') => {
                        self.date += Duration::weeks(1);
                    }
                    KeyCode::Char('W') => {
                        self.date -= Duration::weeks(1);
                    }
                    KeyCode::Char('m') => {
                        self.date += Duration::weeks(4);
                    }
                    KeyCode::Char('M') => {
                        self.date -= Duration::weeks(4);
                    }
                    _ => {}
                },

            InputMode::EditingDate => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::EditingDescription;
                }
                KeyCode::Char(c) => {
                    self.input_date.push(c);
                }
                KeyCode::Backspace => {
                    self.input_date.pop();
                }
                KeyCode::Esc => {
                    self.cancel_input();
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
        let format_hour = parse("[hour][minute]").unwrap();
        let hour_str = self.input_hour.clone();
        let hour = Time::parse(&hour_str, &format_hour).unwrap();

        let format_date = parse("[year]-[month]-[day]").unwrap();
        let date_str = self.input_date.clone();
        let day = Date::parse(&date_str, &format_date).unwrap();

        self.entries.push(Entry {
            description: self.input_desc.clone(),
            hour,
            day,
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
                Constraint::Length(12),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Calendar
        let calendar_block = Block::default()
            .borders(Borders::ALL);
        let inner = calendar_block.inner(chunks[0]);
        calendar_block.render(chunks[0], buf);

        let calendar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(inner);

        let mut events = CalendarEventStore::default();

        for e in &self.entries {
            events.add(
                e.day,
                Style::default().blue().bold(),
                );
        }

        events.add(
            self.date,
            Style::default().yellow().bold(),
            );

        let year = self.date.year();

        let prev_month = self.date.month().previous();
        let next_month = self.date.month().next();

        let prev_month_year = match self.date.month() {
            Month::January => self.date.year() - 1,
            _ => self.date.year()
        };

        let prev_month_date = Date::from_calendar_date(prev_month_year, prev_month, 1).unwrap();
        let next_month_date = Date::from_calendar_date(year, next_month, 1).unwrap();
        
        Monthly::new(prev_month_date, events.clone())
            .block(Block::new().borders(Borders::ALL).padding(Padding::new(1,0,0,0)))
            .show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC)
            .render(calendar_chunks[0], buf);

        Monthly::new(self.date, events.clone())
            .block(Block::new().borders(Borders::ALL).padding(Padding::new(1,0,0,0)))
            .show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC)
            .render(calendar_chunks[1], buf);

        Monthly::new(next_month_date, events.clone())
            .block(Block::new().borders(Borders::ALL).padding(Padding::new(1,0,0,0)))
            .show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC)
            .render(calendar_chunks[2], buf);

        let format = format_description!("[hour]:[minute]");
        let mut rows: Vec<Row> = Vec::new();
        for e in &self.entries {
            if e.day != self.date {
                continue;
            }
            let hour_str = e.hour.format(&format).unwrap();
            rows.push(
                Row::new(vec![
                    Cell::from(hour_str),
                    Cell::from(e.description.as_str()),
                    ]
                )
            );
        }

        let widths = [
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ];

        let table_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(70),
            Constraint::Fill(1),
        ])
            .split(chunks[1])[1];

        Table::new(rows, widths)
            .column_spacing(2)
            .header(
                Row::new(vec!["Hour", "Description"])
                .style(
                    Style::default()
                    .bold()
                    .underlined()
                    )
                .bottom_margin(1),
                )
            .block(
                Block::default()
                .title(" Today ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .padding(Padding::horizontal(1))
            )
            .highlight_symbol(">>")
            .render(table_area, buf);

        let input = match self.input_mode {
            InputMode::EditingDate => {
                Paragraph::new(format!("Date: {}", self.input_date))
            }
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
