use std::env;
use std::fs;
use std::io::Result;
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

use time::Date;
use time::Duration;
use time::Month;
use time::OffsetDateTime;
use time::Time;
// use time::format_description::parse;
use time::macros::format_description;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;

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
// use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::Widget;
use ratatui::widgets::calendar::CalendarEventStore;
use ratatui::widgets::calendar::Monthly;

fn data_dir() -> PathBuf {
    let home = env::var("HOME").unwrap();
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("ratacal")
}

fn open_editor() -> Result<String> {
    let base_dir = data_dir();
    fs::create_dir_all(&base_dir).unwrap();
    let fp = base_dir.join("note.md");
    if !fp.exists() {
        fs::write(&fp, "")?;
    }
    Command::new("nvim")
        .arg(&fp)
        .status()
        .unwrap();
    // TODO Format Content
    let content = fs::read_to_string(&fp)?;
    Ok(content)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn init_terminal(terminal: &mut DefaultTerminal) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

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
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            entries: Vec::new(),
            date: OffsetDateTime::now_utc().date(),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(terminal)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event, terminal)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) {
        match key.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('a') => {
                restore_terminal().unwrap();
                let event: String = open_editor().unwrap();
                init_terminal(terminal).unwrap();
                self.submit_event(event);
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
        }
    }

    fn submit_event(&mut self, event: String) {
        // TODO
        // let format_hour = parse("[hour][minute]").unwrap();
        // let hour_str = self.input_hour.clone();
        // let hour = Time::parse(&hour_str, &format_hour).unwrap();
        //
        // let format_date = parse("[year]-[month]-[day]").unwrap();
        // let date_str = self.input_date.clone();
        // let day = Date::parse(&date_str, &format_date).unwrap();
        //
        // self.entries.push(Entry {
        //     description: self.input_desc.clone(),
        //     hour,
        //     day,
        // });
        // self.entries.sort_by_key(|e| e.hour);
        // self.reset_input();
        todo!();
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
    }
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
