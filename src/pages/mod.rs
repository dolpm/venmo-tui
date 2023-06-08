use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::{Input, TextArea};

pub mod home;
pub mod login;
pub mod me;
pub mod pay;
pub mod qr;
pub mod stories;

const ASCII_TITLE: &'static str = r#"
 __      __                        
 \ \    / /                        
  \ \  / /__ _ __  _ __ ___   ___  
   \ \/ / _ \ '_ \| '_ ` _ \ / _ \ 
    \  /  __/ | | | | | | | | (_) |
     \/ \___|_| |_|_| |_| |_|\___/ 
"#;

#[async_trait]
trait Page {
    // return true if exit
    async fn on_input_event(&mut self, event: Input) -> bool;
    // return true if progress made (skip block for input)
    async fn make_progress(&mut self) -> bool;
    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect);
}

fn inactivate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default().fg(Color::DarkGray)));
}

fn activate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default().fg(Color::Blue)));
}
