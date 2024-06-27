use clap::Parser;
use crossterm::event::{Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::prelude::{Line, Stylize, Terminal, Text};

use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::time::{Duration, Instant};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct Config {
    /// log file
    #[arg(default_value = "../treetest/testdata/dlog0.log")]
    pub filename: String,
}

#[must_use]
pub struct App {
    pub filename: String, // name of the log file to view
    _lines: Vec<String>,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let file = File::open(config.filename.clone()).expect("no such file");
        let buf = BufReader::new(file);
        let lines = buf
            .lines()
            .map(|l| l.expect("couldn't read the file lines"))
            .collect();

        Self {
            filename: config.filename.to_owned(),
            _lines: lines,
        }
    }

    pub fn stylize<'a>(s: String) -> Line<'a> {
        if s.contains("Section") {
            Line::from(s).clone().white()
        }
        else if s.contains("ipsum") {
            Line::from(s).clone().yellow()
        } else {
            Line::from(s).clone().dark_gray()
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let lines: Vec<Line> = self
            ._lines
            .clone()
            .into_iter()
            .map(App::stylize)
            .map(Line::from)
            .collect();
        let log = Paragraph::new(Text::from(lines)).block(Block::bordered().title("Log Lines"));
        frame.render_widget(log, frame.size());
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

    terminal.draw(|frame| app.draw(frame))?;

    let mut debounce: Option<Instant> = None;

    loop {
        let timeout = debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
        if crossterm::event::poll(timeout)? {
            let update = match crossterm::event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => false,
                },
                Event::Resize(_, _) => true,
                _ => false,
            };
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
        }
        if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
            terminal.draw(|frame| {
                app.draw(frame);
            })?;

            debounce = None;
        }
    }
}
