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
    widgets::{Block, List, ListDirection, ListItem, Paragraph, Tabs, Wrap},
    DefaultTerminal, Frame,
};
use rust_http::{client::HttpClient, http::{HttpMethod, HttpRequest, HttpResponse, HTTP_METHODS}};

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    url_input: String,
    headers_input: String,
    body_input: String,
    /// Position of cursor in the editor area.
    character_index: usize,

    error_message: Option<String>,

    client: HttpClient,

    responses: Vec<HttpResponse>,

    method_index: usize,

    input_order: Vec<InputMode>,
    input_index: usize, 
}

#[derive(PartialEq)]
enum InputMode {
    EditingUrl,
    EditingHeaders,
    EditingBody,
    EditingMethod,
}

impl App {
    pub fn new(client: HttpClient, server_addr: String) -> Self {
        let empty_string = "".to_string();

        Self {
            input_order: vec![InputMode::EditingMethod, InputMode::EditingUrl, InputMode::EditingHeaders, InputMode::EditingBody],
            input_index: 3,
            character_index: 0,
            error_message: None,
            url_input: server_addr,
            headers_input: empty_string.clone(),
            body_input: empty_string,
            client,
            responses: vec![],
            method_index: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        if *self.get_input_mode() == InputMode::EditingMethod {
            self.method_index =  (self.method_index + HTTP_METHODS.len() - 1) % HTTP_METHODS.len();
            return
        }
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        if *self.get_input_mode() == InputMode::EditingMethod {
            self.method_index =  (self.method_index + HTTP_METHODS.len() + 1) % HTTP_METHODS.len();
            return
        }
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn get_input_mode(&self) -> &InputMode {
        self.input_order.get(self.input_index).unwrap()
    }

    fn get_current_input_mut(&mut self) -> &mut String {
        match self.get_input_mode() {
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
        match self.get_input_mode() {
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
            let before_char_to_delete = self.get_current_input().chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.get_current_input().chars().skip(current_index);

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

    fn send_req(&mut self) {
        let req = HttpRequest {
            method: HTTP_METHODS[self.method_index].clone(),
            endpoint: self.url_input.clone(),
            headers: vec![], // TODO
            body: self.body_input.clone(),
        };

        match self.client.send(req, &self.url_input) {
            Ok(res) => self.responses.insert(0,res),
            Err(e) => println!("{}", e),
        }
    }

    pub fn move_input_mode_up(&mut self) {
        let index_shift = self.input_index + self.input_order.len() - 1;
        self.input_index = index_shift % self.input_order.len();
    }
    pub fn move_input_mode_down(&mut self) {
        let index_shift = self.input_index + self.input_order.len() + 1;
        self.input_index = index_shift % self.input_order.len();
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => self.send_req(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Down => self.move_input_mode_down(),
                        KeyCode::Up => self.move_input_mode_up(),
                        _ => {},
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [method_area, url_area, header_area, big_area] = vertical.areas(frame.area());

        let horizontal = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Min(1),
        ]);
        let [body_area, response_area] = horizontal.areas(big_area);

        let methods = Tabs::new(HTTP_METHODS.iter().map(|method| format!("{:.?}", method)))
            .block(Block::bordered().title("Methods"))
            .select(self.method_index)
            .style(Style::default().fg(if *self.get_input_mode() == InputMode::EditingMethod {Color::Yellow} else {Color::White}));
        frame.render_widget(methods, method_area);

        let url_input = Paragraph::new(self.url_input.as_str())
            .style(Style::default().fg(if *self.get_input_mode() == InputMode::EditingUrl {Color::Yellow} else {Color::White}))
            .block(Block::bordered().title("Input"));
        frame.render_widget(url_input, url_area);

        let headers_input = Paragraph::new(self.headers_input.as_str())
            .style(Style::default().fg(if *self.get_input_mode() == InputMode::EditingHeaders {Color::Yellow} else {Color::White}))
            .block(Block::bordered().title("Headers"));
        frame.render_widget(headers_input, header_area);

        let body_input = Paragraph::new(self.body_input.as_str())
            .style(Style::default().fg(if *self.get_input_mode() == InputMode::EditingBody {Color::Yellow} else {Color::White}))
            .block(Block::bordered().title("Body"))
            .wrap(Wrap {trim: true});
        frame.render_widget(body_input, body_area);
        
        let response = List::new(self.responses.iter().map(|res| format!("{:#?}\n---------------------------------", res)))
            .block(Block::bordered().title("Responses"));
        frame.render_widget(response, response_area);
    }
}