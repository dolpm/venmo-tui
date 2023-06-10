use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

use crate::{api::Api, types::FundingInstrument};

use super::{activate, centered_rect, home::StatefulList, inactivate, Page};

#[derive(Copy, Clone, PartialEq)]
enum Field {
    Unset,
    Amount,
    Handle,
    Note,
    Pay,
    Request,
}

struct PaymentSourcePopup {
    items: StatefulList<FundingInstrument>,
}

impl PaymentSourcePopup {
    fn new(sources: Vec<FundingInstrument>) -> PaymentSourcePopup {
        PaymentSourcePopup {
            items: StatefulList::with_items(sources),
        }
    }
}

pub struct PayPage<'a> {
    selected: Field,
    waiting_for_submit: bool,
    show_popup: bool,
    popup_items: Vec<ListItem<'a>>,
    amount: TextArea<'a>,
    handle: TextArea<'a>,
    note: TextArea<'a>,
    send: Paragraph<'a>,
    recv: Paragraph<'a>,
    popup: PaymentSourcePopup,
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
            popup: PaymentSourcePopup::new(vec![]),
            waiting_for_submit: false,
            show_popup: false,
            popup_items: vec![],
            selected: Field::Unset,
        };

        v.amount
            .set_block(Block::default().borders(Borders::ALL).title("$"));
        v.handle
            .set_block(Block::default().borders(Borders::ALL).title("Username"));
        v.note
            .set_block(Block::default().borders(Borders::ALL).title("Note"));

        inactivate(&mut v.amount);
        inactivate(&mut v.handle);
        inactivate(&mut v.note);

        v
    }

    fn validate_amount(&mut self) {
        if let Err(_) = self.amount.lines()[0].parse::<f64>() {
            self.amount.set_style(Style::default().fg(Color::LightRed));
            self.amount.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("ERROR: provided value must be a number"),
            );
        } else {
            self.amount.set_style(Style::default().fg(Color::Blue));
            self.amount
                .set_block(Block::default().borders(Borders::ALL).title("$"));
        }
    }

    async fn validate_username(&mut self) {
        if let Err(_) = self.api.fetch_user_id(&self.handle.lines()[0]).await {
            self.handle.set_style(Style::default().fg(Color::LightRed));
            self.handle.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("ERROR: username not valid"),
            );
        } else {
            inactivate(&mut self.handle);
        }
    }

    fn render_payment_source_popup(
        &mut self,
        f: &mut Frame<'_, CrosstermBackend<StdoutLock<'_>>>,
        area: Rect,
    ) {
        let block = Block::default().style(Style::default().bg(Color::White));
        let area = centered_rect(40, 60, area);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
        let list = List::new(self.popup_items.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Funding Source")
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_stateful_widget(list, area, &mut self.popup.items.state);
    }
}

#[async_trait]
impl<'a> Page for PayPage<'a> {
    async fn on_input_event(&mut self, event: Input) -> bool {
        if self.show_popup {
            match event {
                Input { key: Key::Down, .. } => {
                    self.popup.items.next();
                }
                Input { key: Key::Up, .. } => {
                    self.popup.items.previous();
                }
                Input { key: Key::Esc, .. } => {
                    // reset selection if re-opened
                    self.popup.items.state.select(Some(0));
                    self.show_popup = false;
                }
                _ => {}
            }

            return false;
        }

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
                    Field::Pay => {
                        self.send = self.send.clone().style(Style::default());
                        activate(&mut self.note);
                        Field::Note
                    }
                    Field::Request => {
                        self.recv = self.recv.clone().style(Style::default());
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
                if self.selected == Field::Request && self.waiting_for_submit == false {
                    self.waiting_for_submit = true;
                }

                if self.selected == Field::Pay {
                    self.popup.items.items = self.api.get_funding_instruments().await.expect(":(");
                    self.popup_items = self
                        .popup
                        .items
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            ListItem::new(Spans::from(format!(
                                "{}. {} ({})",
                                i + 1,
                                v.name.clone(),
                                v.instrument_type.clone()
                            )))
                            .style(Style::default().fg(Color::Black))
                        })
                        .collect();
                    self.show_popup = true;
                }

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
            k => match self.selected {
                Field::Amount => {
                    self.amount.input(k.clone());
                    self.validate_amount();
                }
                Field::Handle => {
                    self.handle.input(k.clone());
                }
                Field::Note => {
                    self.note.input(k.clone());
                }

                _ => {}
            },
        }

        self.selected == Field::Unset
    }
    async fn make_progress(&mut self) -> bool {
        if self.selected == Field::Unset {
            self.selected = Field::Amount;
            activate(&mut self.amount);
        }

        if self.waiting_for_submit {
            let amount_in_cents = (self.amount.lines()[0]
                .parse::<f32>()
                .expect("failed to parse to u32")
                * 100.0) as u32;

            let user_id = self
                .api
                .fetch_user_id(&self.handle.lines()[0])
                .await
                .expect("don't fail rn");

            let instrument = None;

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
                    instrument,
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

        if self.show_popup {
            self.render_payment_source_popup(f, area);
        }
    }
}
