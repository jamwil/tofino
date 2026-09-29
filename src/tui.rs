use ratatui::{DefaultTerminal, Frame};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TuiError;

impl Error for TuiError {}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurrend!") // TODO
    }
}

pub fn tui(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("Hello, world!", frame.area());
}
