use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use std::io;

fn handle_mouse_event(event: MouseEvent) {
    match event.kind {
        MouseEventKind::Down(button) => {
            println!(
                "{:?} Button Down at {}x{}\r",
                button, event.column, event.row
            )
        }
        MouseEventKind::Up(button) => {
            println!(
                "{:?} Button Up   at {}x{}\r",
                button, event.column, event.row
            )
        }
        _ => {
            println!("{:?} at {}x{}\r", event.kind, event.column, event.row)
        }
    }
}

fn handle_key_event(event: KeyEvent) -> bool {
    println!("{:?}\r", event);

    // return true if quitting
    event.code == KeyCode::Char('q')
}

fn print_events() -> std::io::Result<()> {
    println!("Press q to exit\r");
    loop {
        match read()? {
            Event::FocusGained => println!("Focus Gained\r"),
            Event::FocusLost => println!("Focus Lost\r"),
            Event::Key(event) => {
                if handle_key_event(event) {
                    break;
                }
            }
            Event::Mouse(event) => handle_mouse_event(event),
            Event::Paste(data) => println!("{}", data),
            Event::Resize(width, height) => {
                print!("New size: {}x{}\r\n", width, height);
            }
        }
    }
    Ok(())
}

fn main() {
    enable_raw_mode().expect("Couldn't enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("Couldn't EnterAlternateScreen or EnableMouseCapture");

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .expect("Couldn't clear terminal");
    if let Err(e) = print_events() {
        println!("Error: {}", e);
    }

    disable_raw_mode().expect("Couldn't disable raw mode");
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)
        .expect("Couldn't LeaveAlternateScreen or DisableMouseCapture");
}
