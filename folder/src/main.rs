use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

struct App {
    paragraphs: Vec<Vec<String>>,
    collapsed: Vec<bool>,
    selected: usize,
    nav_selected: usize,
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

        let collapsed = vec![true; paragraphs.len()];

        App {
            paragraphs,
            collapsed,
            selected: 0,
            nav_selected: 0,
        }
    }

    fn toggle(&mut self) {
        if let Some(val) = self.collapsed.get_mut(self.selected) {
            *val = !*val;
        }
    }

    fn next(&mut self) {
        if self.nav_selected < self.paragraphs.len() - 1 {
            self.nav_selected += 1;
        }
    }

    fn prev(&mut self) {
        if self.nav_selected > 0 {
            self.nav_selected -= 1;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = r#"
This is the first paragraph.
It has two lines.

Second paragraph here, also with
two lines.

A final short paragraph.
"#;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the screen before starting
    terminal.clear()?;

    let mut app = App::new(text);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(1)])
                .split(size);

            // Navigation bar on the left
            let items: Vec<ListItem> = app
                .paragraphs
                .iter()
                .enumerate()
                .map(|(i, para)| {
                    let style = if i == app.nav_selected {
                        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    ListItem::new(para[0].clone()).style(style)
                })
                .collect();

            let nav =
                List::new(items).block(Block::default().title("Paragraphs").borders(Borders::ALL));

            f.render_widget(nav, chunks[0]);

            // Main content area on the right
            let visible_lines = app.visible_lines();
            let mut text = Text::default();

            for (para_index, line) in visible_lines {
                let is_selected = para_index == app.selected;
                let style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                };
                text.push_line(Line::styled(line, style));
            }

            let content_block = Block::default()
                .title("Content (q: quit, arrows: nav, Enter: select, Space: toggle)")
                .borders(Borders::ALL);

            let paragraph = Paragraph::new(text).block(content_block);
            f.render_widget(paragraph, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.prev(),
                    KeyCode::Enter => app.select_nav(),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
