use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::{error::Error, io};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Left,
    Right,
}

struct App {
    paragraphs: Vec<Vec<String>>,
    collapsed: Vec<bool>,
    selected: usize,
    nav_selected: usize,
    active_pane: Pane,
    nav_scroll_offset: usize,
}

impl App {
    fn new(text: &str) -> Self {
        let mut paragraphs = Vec::new();
        let mut current = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                if !current.is_empty() {
                    paragraphs.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(line.to_string());
            }
        }
        if !current.is_empty() {
            paragraphs.push(current);
        }

        let collapsed = vec![false; paragraphs.len()];

        App {
            paragraphs,
            collapsed,
            selected: 0,
            nav_selected: 0,
            active_pane: Pane::Left,
            nav_scroll_offset: 0,
        }
    }

    fn toggle(&mut self) {
        match self.active_pane {
            Pane::Left => {
                if let Some(val) = self.collapsed.get_mut(self.nav_selected) {
                    *val = !*val;
                }
            }
            Pane::Right => {
                if let Some(val) = self.collapsed.get_mut(self.selected) {
                    *val = !*val;
                }
            }
        }
    }

    fn next(&mut self) {
        match self.active_pane {
            Pane::Left => {
                if self.nav_selected + 1 < self.paragraphs.len() {
                    self.nav_selected += 1;
                    let height = 10; // placeholder, will be updated in run_app
                    // Adjust nav_scroll_offset to keep nav_selected visible
                    if self.nav_selected >= self.nav_scroll_offset + height {
                        self.nav_scroll_offset = self.nav_selected - height + 1;
                    }
                }
            }
            Pane::Right => {
                if self.selected + 1 < self.paragraphs.len() {
                    self.selected += 1;
                }
            }
        }
    }

    fn prev(&mut self) {
        match self.active_pane {
            Pane::Left => {
                if self.nav_selected > 0 {
                    self.nav_selected -= 1;
                    // Adjust nav_scroll_offset to keep nav_selected visible
                    if self.nav_selected < self.nav_scroll_offset {
                        self.nav_scroll_offset = self.nav_selected;
                    }
                }
            }
            Pane::Right => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
        }
    }

    fn select_nav(&mut self) {
        self.selected = self.nav_selected;
    }

    fn visible_lines(&self) -> Vec<(usize, String)> {
        let mut lines = Vec::new();
        for (i, para) in self.paragraphs.iter().enumerate() {
            if self.collapsed[i] {
                lines.push((i, para[0].clone()));
            } else {
                for line in para {
                    lines.push((i, line.clone()));
                }
            }
        }
        lines
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let text = "\
This is paragraph one.
It has multiple lines.
Line three of paragraph one.

Paragraph two starts here.
It also has multiple lines.

Third paragraph is here.
Single line paragraph.";

    let mut app = App::new(text);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(area);

            // Left pane: navigation list
            let mut nav_text = Text::default();
            for (i, para) in app.paragraphs.iter().enumerate() {
                let style = if i == app.nav_selected && app.active_pane == Pane::Left {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let prefix = if app.collapsed[i] { "[+]" } else { "[-]" };
                let first_line = &para[0];
                let line = format!("{} {}", prefix, first_line);

                nav_text.push_line(Line::styled(line, style));
            }

            let nav_block = Block::default()
                .borders(Borders::ALL)
                .title("Navigation")
                .border_style(if app.active_pane == Pane::Left {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            let nav_height = chunks[0].height.saturating_sub(2) as usize; // account for borders
            if app.nav_selected >= app.nav_scroll_offset + nav_height {
                app.nav_scroll_offset = app.nav_selected - nav_height + 1;
            } else if app.nav_selected < app.nav_scroll_offset {
                app.nav_scroll_offset = app.nav_selected;
            }

            let nav_paragraph = Paragraph::new(nav_text)
                .block(nav_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .scroll((app.nav_scroll_offset as u16, 0));
            f.render_widget(nav_paragraph, chunks[0]);

            // Right pane: render all visible lines, highlight those corresponding to selected paragraph
            let right_area = chunks[1];
            let visible_lines = app.visible_lines();
            let mut content_text = Text::default();
            let mut selected_line_idx = None;
            for (idx, (para_idx, line)) in visible_lines.iter().enumerate() {
                let style = if *para_idx == app.selected {
                    if selected_line_idx.is_none() {
                        selected_line_idx = Some(idx);
                    }
                    if app.active_pane == Pane::Right {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default()
                };
                content_text.push_line(Line::styled(line.clone(), style));
            }

            let content_block = Block::default()
                .borders(Borders::ALL)
                .title("Content")
                .border_style(if app.active_pane == Pane::Right {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            // Compute the scroll offset so that the first line of the selected paragraph is visible
            let height = right_area.height.saturating_sub(2) as usize; // account for borders
            let selected_line = selected_line_idx.unwrap_or(0);
            let mut scroll = 0;
            if selected_line >= height {
                scroll = selected_line - height / 2;
            }
            let content_paragraph = Paragraph::new(content_text)
                .block(content_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .scroll((scroll as u16, 0));
            f.render_widget(content_paragraph, right_area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.prev();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.active_pane = Pane::Left;
                        app.nav_selected = app.selected; // sync selection
                    }
                    KeyCode::Right | KeyCode::Char('l') => app.active_pane = Pane::Right,
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Enter => {
                        if app.active_pane == Pane::Left {
                            app.select_nav();
                            app.active_pane = Pane::Right;
                        }
                    }
                    KeyCode::Home => {
                        if app.active_pane == Pane::Right {
                            app.selected = 0;
                        }
                    }
                    KeyCode::End => {
                        if app.active_pane == Pane::Right {
                            app.selected = if app.paragraphs.is_empty() {
                                0
                            } else {
                                app.paragraphs.len() - 1
                            };
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
