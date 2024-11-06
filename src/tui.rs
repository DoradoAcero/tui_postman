//! # [Ratatui] User Input example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

// A simple example demonstrating how to handle user input. This is a bit out of the scope of
// the library as it does not provide any input handling out of the box. However, it may helps
// some to get started.
//
// This is a very simple example:
//   * An input box always focused. Every character you type is registered here.
//   * An entered character is inserted at the cursor position.
//   * Pressing Backspace erases the left character before the cursor position
//   * Pressing Enter pushes the current input in the history of previous messages. **Note: ** as
//   this is a relatively simple example unicode characters are unsupported and their use will
// result in undefined behaviour.
//
// See also https://github.com/rhysd/tui-textarea and https://github.com/sayanarijit/tui-input/

use std::collections::HashMap;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use rust_http::client::HttpClient;

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    url_input: String,
    headers_input: String,
    body_input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,

    error_message: Option<String>,

    client: HttpClient,
}

enum InputMode {
    Normal,
    EditingUrl,
    EditingHeaders,
    EditingBody,
}

impl App {
    pub fn new(client: HttpClient, server_addr: String) -> Self {
        let empty_string = "".to_string();

        Self {
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
            error_message: None,
            url_input: server_addr,
            headers_input: empty_string.clone(),
            body_input: empty_string,
            client,
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

    fn get_current_input_mut(&mut self) -> &mut String {
        match self.input_mode {
            InputMode::EditingBody => {
                &mut self.body_input
            },
            InputMode::EditingHeaders => {
                &mut self.headers_input
            },
            InputMode::EditingUrl => {
                &mut self.url_input
            },
            _ => panic!("Should never get here"),
        }
    }

    fn get_current_input(&self) -> &String {
        match self.input_mode {
            InputMode::EditingBody => {
                &self.body_input
            },
            InputMode::EditingHeaders => {
                &self.headers_input
            },
            InputMode::EditingUrl => {
                &self.url_input
            },
            _ => panic!("Should never get here"),
        }
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.get_current_input_mut().insert(index, new_char);
        self.error_message = None;
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        let input = self.get_current_input();
        input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.get_current_input_mut().chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.get_current_input_mut().chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            *self.get_current_input_mut() = before_char_to_delete.chain(after_char_to_delete).collect();
            self.error_message = None;
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.get_current_input().chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, error_area, input_area, messages_area] = vertical.areas(frame.area());
        let horo_help = Layout::horizontal([
            Constraint::Min(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
        ]);

        let [help_message_area, basic_guess_area, complex_guess_area, complex_guess_area_2] = horo_help.areas(help_area);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };

        if self.won {
            frame.render_widget(Text::from("CONGRATULATIONS!!! You guessed the correct word").patch_style(Style::default().fg(Color::Green)), error_area)
        } else {
            match &self.error_message {
                Some(e_msg) => frame.render_widget(Text::from(e_msg.clone()).patch_style(Style::default().fg(Color::Red)), error_area),
                _ => (),
            };
        }
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_message_area);

        let areas = vec![basic_guess_area, complex_guess_area, complex_guess_area_2];
        for (i, area) in areas.into_iter().enumerate() {
            frame.render_widget(self.ai_guesses.get(i).unwrap(), area);
        }

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                let content = self.style_word(m.clone());
                ListItem::new(content.clone())
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Guesses"));
        frame.render_widget(messages, messages_area);
    }
}