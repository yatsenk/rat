use crate::server::{ClientHandle, spawn_client};

pub struct TabsState<'titles> {
    pub titles: Vec<&'titles str>,
    pub index: usize,
}

impl<'titles> TabsState<'titles> {
    pub const fn new(titles: Vec<&'titles str>) -> Self {
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

pub struct App<'title> {
    pub title: &'title str,
    pub input: String,
    pub tabs: TabsState<'title>,
    pub character_index: usize,
    pub messages: Vec<String>,
    pub instructions: Vec<String>,
    pub client_addr: String,
    pub logged_keys: String,
    pub client: ClientHandle,
}

impl<'title> App<'title> {
    pub fn new(title: &'title str) -> Self {
        Self {
            title,
            input: String::new(),
            tabs: TabsState::new(vec!["Client", "View", "Input"]),
            messages: Vec::new(),
            instructions: Vec::new(),
            character_index: 0,
            client_addr: String::new(),
            logged_keys: String::new(),
            client: spawn_client("127.0.0.1:7878"),
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
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

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn submit_instructions(&mut self) {
        if self.client.app_sender.blocking_send(
            self.input.clone().as_bytes().to_vec()
        ).is_err() {
            return;
        }

        self.instructions.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    pub fn on_tick(&mut self) { 

    }

}

