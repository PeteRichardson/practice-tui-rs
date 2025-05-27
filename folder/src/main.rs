use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

struct App {
    paragraphs: Vec<Vec<String>>,
    collapsed: Vec<bool>,
    selected: usize,
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
        }
    }

    fn toggle(&mut self) {
        if let Some(val) = self.collapsed.get_mut(self.selected) {
            *val = !*val;
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1).min(self.paragraphs.len() - 1);
    }

    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Flattens the visible lines for rendering, keeping track of which paragraph they belong to.
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
    terminal.clear()?;

    let mut app = App::new(text);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let lines = app.visible_lines();

            let mut text = Text::default();

            for (para_index, line) in lines {
                let is_selected = para_index == app.selected;
                let style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                };
                text.push_line(Line::styled(line, style));
            }

            let block = Block::default()
                .title("Collapsible Text Viewer (q: quit, up/down, space to toggle)")
                .borders(Borders::ALL);

            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, size);
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
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
