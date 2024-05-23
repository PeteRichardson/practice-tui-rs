use clap::Parser;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::backend::Backend;
use ratatui::layout::Position;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};
use std::time::{Duration, Instant};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct Config {
    /// log file
    #[arg(default_value = "testdata/dlog0.log")]
    pub filename: String,
}

#[must_use]
pub struct App {
    pub filename: String, // name of the log file to view
    pub state: TreeState<&'static str>,
    items: Vec<TreeItem<'static, &'static str>>,
    _lines: Vec<String>,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let file = File::open(config.filename.clone()).expect("no such file");
        let buf = BufReader::new(file);
        let lines = buf
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();

        let mut app = Self {
            filename: config.filename.to_owned(),
            state: TreeState::default(),
            items: vec![
                TreeItem::new_leaf("Section 1", "Section 1"),
                TreeItem::new(
                    "Section 2",
                    "Section 2",
                    vec![
                        TreeItem::new_leaf("Section 2.1", "Section 2.1"),
                        TreeItem::new_leaf("Section 2.2", "Section 2.2"),
                    ],
                )
                .expect("all item identifiers are unique"),
                TreeItem::new_leaf("Section 3", "Section 3"),
            ],
            _lines: lines,
        };
        app.state.select(vec!["a"]);
        app
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title("Tree Widget")
                    .title_bottom(format!("{:?}", self.state)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("");
        frame.render_stateful_widget(widget, area, &mut self.state);
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
                    KeyCode::Char('\n' | ' ') => app.state.toggle_selected(),
                    KeyCode::Left => {
                        // Always want there to be a selection, so don't do anything
                        // if a first-level item is selected and it's not opened.
                        if app.state.selected().len() == 1
                            && !app.state.opened().contains(app.state.selected())
                        {
                            false
                        } else {
                            app.state.key_left()
                        }
                    }

                    KeyCode::Right => app.state.key_right(),
                    KeyCode::Down => app.state.key_down(),
                    KeyCode::Up => app.state.key_up(),
                    KeyCode::Esc => app.state.select_first(),
                    KeyCode::Home => app.state.select_first(),
                    KeyCode::End => app.state.select_last(),
                    KeyCode::PageDown => app.state.scroll_down(3),
                    KeyCode::PageUp => app.state.scroll_up(3),
                    _ => false,
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.state.scroll_down(1),
                    MouseEventKind::ScrollUp => app.state.scroll_up(1),
                    MouseEventKind::Down(_button) => {
                        app.state.click_at(Position::new(mouse.column, mouse.row))
                    }
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
