use core::panic;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use venmo_tui::api::Api;
use venmo_tui::pages::home::draw_home_page;
use venmo_tui::pages::login::draw_login_page;

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut api = Box::leak(Box::new(match Api::new().await {
        Err(e) => {
            panic!("{e}");
        }
        Ok(v) => v,
    }));

    let logged_in = api.logged_in().await;

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    if !logged_in {
        if let None = draw_login_page(&mut term, &mut api).await? {
            disable_raw_mode()?;
            crossterm::execute!(
                term.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            term.show_cursor()?;
            return Ok(());
        }
    }

    // load identity before drawing home page
    api.get_profile().await.expect("failed to get profile");

    draw_home_page(&mut term, api).await?;

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}
