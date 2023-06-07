use std::io;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use tui_textarea::{Input, Key, TextArea};

use crate::{api::Api, types::LoginResponse};

use super::{activate, inactivate, ASCII_TITLE};

enum LoginField {
    Username,
    Password,
    Login,
}

pub async fn draw_login_page<T>(
    term: &mut Terminal<T>,
    api: &mut Api,
) -> io::Result<Option<LoginResponse>>
where
    T: Backend,
{
    let [mut username, mut password] = [TextArea::default(), TextArea::default()];

    // set titles
    {
        username.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username/Phone"),
        );
        password.set_block(Block::default().borders(Borders::ALL).title("Password"));
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        );

    let text = ASCII_TITLE
        .lines()
        .skip(1)
        .map(|l| Spans::from(Span::styled(l, Style::default())))
        .collect::<Vec<_>>();

    let mut selected: LoginField = LoginField::Username;

    activate(&mut username);
    inactivate(&mut password);

    let title = Paragraph::new(text.clone())
        .style(Style::default().fg(Color::Blue))
        .block(Block::default())
        .alignment(Alignment::Center);

    let login_btn_block = Block::default();
    let mut login_btn_text = Paragraph::new(Text::from("Login"))
        .block(login_btn_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    loop {
        term.draw(|f| {
            let chunks = layout.split(f.size());
            f.render_widget(title.clone(), chunks[0]);
            let u_widget = username.widget();
            f.render_widget(u_widget, chunks[1]);
            let p_widget = password.widget();
            f.render_widget(p_widget, chunks[2]);

            f.render_widget(login_btn_text.clone(), chunks[3]);
        })?;
        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            Input { key: Key::Down, .. } => match selected {
                LoginField::Username => {
                    inactivate(&mut username);
                    activate(&mut password);
                    selected = LoginField::Password;
                }
                LoginField::Password => {
                    inactivate(&mut password);
                    login_btn_text = login_btn_text.style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                    selected = LoginField::Login;
                }
                _ => {}
            },
            Input { key: Key::Up, .. } => match selected {
                LoginField::Password => {
                    inactivate(&mut password);
                    activate(&mut username);
                    selected = LoginField::Username;
                }
                LoginField::Login => {
                    login_btn_text = login_btn_text.style(Style::default());
                    activate(&mut password);
                    selected = LoginField::Password;
                }
                _ => {}
            },
            Input {
                key: Key::Enter, ..
            } => match selected {
                LoginField::Username => {
                    inactivate(&mut username);
                    activate(&mut password);
                    selected = LoginField::Password;
                }
                LoginField::Password => {
                    inactivate(&mut password);
                    login_btn_text = login_btn_text.style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                    selected = LoginField::Login;
                }
                LoginField::Login => {
                    match api.login(&username.lines()[0], &password.lines()[0]).await {
                        Err(e) => {
                            // draw error
                            panic!("{}", e);
                        }
                        Ok(v) => return Ok(Some(v)),
                    }
                }
            },
            input => {
                match selected {
                    LoginField::Username => username.input(input),
                    LoginField::Password => password.input(input),
                    LoginField::Login => false,
                };
            }
        }
    }

    Ok(None)
}
