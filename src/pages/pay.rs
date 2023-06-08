use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

use crate::api::Api;

use super::{activate, inactivate, Page};

#[derive(Copy, Clone, PartialEq)]
enum Field {
    Unset,
    Amount,
    Handle,
    Note,
    Pay,
    Request,
}

pub struct PayPage<'a> {
    selected: Field,
    waiting_for_submit: bool,
    amount: TextArea<'a>,
    handle: TextArea<'a>,
    note: TextArea<'a>,
    send: Paragraph<'a>,
    recv: Paragraph<'a>,
    api: &'a mut Api,
}

impl<'a> PayPage<'a> {
    pub fn new(api: &'a mut Api) -> Self {
        let mut v = Self {
            api,
            amount: TextArea::default(),
            handle: TextArea::default(),
            note: TextArea::default(),
            send: Paragraph::new(Text::from("Pay")).alignment(Alignment::Right),
            recv: Paragraph::new(Text::from("Request")).alignment(Alignment::Left),
            waiting_for_submit: false,
            selected: Field::Unset,
        };

        v.amount
            .set_block(Block::default().borders(Borders::ALL).title("$"));
        v.handle.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Name, @username, email, phone"),
        );
        v.note
            .set_block(Block::default().borders(Borders::ALL).title("Note"));

        inactivate(&mut v.amount);
        inactivate(&mut v.handle);
        inactivate(&mut v.note);

        v
    }
}

#[async_trait]
impl<'a> Page for PayPage<'a> {
    async fn on_input_event(&mut self, event: Input) -> bool {
        match event {
            Input { key: Key::Down, .. } => {
                self.selected = match self.selected {
                    Field::Amount => {
                        inactivate(&mut self.amount);
                        activate(&mut self.handle);
                        Field::Handle
                    }
                    Field::Handle => {
                        inactivate(&mut self.handle);
                        activate(&mut self.note);
                        Field::Note
                    }
                    Field::Note => {
                        self.send = self.send.clone().style(
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        );
                        inactivate(&mut self.note);
                        Field::Pay
                    }
                    f => f,
                };
            }
            Input { key: Key::Up, .. } => {
                self.selected = match self.selected {
                    Field::Handle => {
                        inactivate(&mut self.handle);
                        activate(&mut self.amount);
                        Field::Amount
                    }
                    Field::Note => {
                        inactivate(&mut self.note);
                        activate(&mut self.handle);
                        Field::Handle
                    }
                    Field::Pay | Field::Request => {
                        activate(&mut self.note);
                        Field::Note
                    }
                    f => f,
                }
            }
            Input { key: Key::Left, .. } => {
                self.selected = match self.selected {
                    Field::Request => {
                        self.recv = self.recv.clone().style(Style::default());
                        self.send = self.send.clone().style(
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        );
                        Field::Pay
                    }
                    Field::Amount => {
                        inactivate(&mut self.amount);
                        Field::Unset
                    }
                    Field::Handle => {
                        inactivate(&mut self.handle);
                        Field::Unset
                    }
                    Field::Note => {
                        inactivate(&mut self.note);
                        Field::Unset
                    }
                    Field::Pay => {
                        self.send = self.send.clone().style(Style::default());
                        Field::Unset
                    }
                    _ => Field::Unset,
                }
            }
            Input {
                key: Key::Right, ..
            } => {
                self.selected = match self.selected {
                    Field::Pay => {
                        self.send = self.send.clone().style(Style::default());
                        self.recv = self.recv.clone().style(
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        );
                        Field::Request
                    }
                    f => f,
                }
            }
            Input {
                key: Key::Enter, ..
            } => {
                self.waiting_for_submit = match self.selected {
                    Field::Pay | Field::Request => true,
                    _ => false,
                };

                self.selected = match self.selected {
                    Field::Amount => {
                        inactivate(&mut self.amount);
                        activate(&mut self.handle);
                        Field::Handle
                    }
                    Field::Handle => {
                        inactivate(&mut self.handle);
                        activate(&mut self.note);
                        Field::Note
                    }
                    f => f,
                }
            }
            Input { key: Key::Esc, .. } => return true,
            k => {
                let _ = match self.selected {
                    Field::Amount => self.amount.input(k.clone()),
                    Field::Handle => self.handle.input(k.clone()),
                    Field::Note => self.note.input(k.clone()),
                    _ => false,
                };
            }
        }

        self.selected == Field::Unset
    }
    async fn make_progress(&mut self) -> bool {
        if self.selected == Field::Unset {
            self.selected = Field::Amount;
            activate(&mut self.amount);
        }

        if self.waiting_for_submit {
            let amount_in_cents = self.amount.lines()[0]
                .parse::<u32>()
                .expect("failed to parse to u32")
                * 100;

            let user_id = self
                .api
                .fetch_user_id(&self.handle.lines()[0])
                .await
                .expect("don't fail rn");

            self.api
                .submit_payment(
                    amount_in_cents,
                    &self.note.lines()[0],
                    &user_id,
                    match self.selected {
                        Field::Pay => crate::api::PaymentType::Pay,
                        Field::Request => crate::api::PaymentType::Request,
                        _ => panic!("not possible!"),
                    },
                )
                .await
                .expect("don't fail rn");

            self.waiting_for_submit = false;
            return true;
        }

        false
    }

    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .margin(2)
            .split(area);

        {
            let a_widget = self.amount.widget();
            f.render_widget(a_widget, chunks[0]);
            let h_widget = self.handle.widget();
            f.render_widget(h_widget, chunks[1]);
            let n_widget = self.note.widget();
            f.render_widget(n_widget, chunks[2]);

            let btn_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(47),
                        Constraint::Length(6),
                        Constraint::Percentage(47),
                    ]
                    .as_ref(),
                )
                .split(chunks[3]);

            f.render_widget(self.send.clone(), btn_layout[0]);
            f.render_widget(self.recv.clone(), btn_layout[2]);
        }

        let block = Block::default()
            .title("Pay & Request")
            .borders(Borders::ALL);
        f.render_widget(block, area);
    }
}
