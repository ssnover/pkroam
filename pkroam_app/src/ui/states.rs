use crate::app::AppEvent;
use crate::types::{Game, GameSave};
use crossterm::event::{KeyCode, KeyEvent};
use num_traits::FromPrimitive;
use pkroam::save::SaveFile;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    text::Line,
    Terminal,
};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum UiState {
    SaveSelection(SaveSelection),
    NewSaveEntry(NewSaveEntry),
}

impl UiState {
    pub fn handle_key(&mut self, key_event: &KeyEvent, event_sender: &Sender<AppEvent>) {
        match self {
            UiState::SaveSelection(data) => data.handle_key(key_event, event_sender),
            UiState::NewSaveEntry(data) => data.handle_key(key_event, event_sender),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            UiState::SaveSelection(data) => data.draw(terminal),
            UiState::NewSaveEntry(data) => data.draw(terminal),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SaveSelection {
    saves: Vec<GameSave>,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    highlighted_row: u16,
}

impl SaveSelection {
    pub fn new(saves: Vec<GameSave>) -> Self {
        Self {
            saves,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            highlighted_row: 0,
        }
    }

    fn handle_key(&mut self, key_event: &KeyEvent, event_sender: &Sender<AppEvent>) {
        match key_event.code {
            KeyCode::Down => {
                self.highlighted_row =
                    std::cmp::min(self.saves.len() as u16 + 2 - 1, self.highlighted_row + 1);
            }
            KeyCode::Up => {
                self.highlighted_row = std::cmp::max(0, self.highlighted_row - 1);
            }
            KeyCode::Enter => {
                let highlighted_row = self.highlighted_row as usize;
                if (0..self.saves.len()).contains(&highlighted_row) {
                    // Selected a save file
                    log::debug!("Connecting to a save file");
                    let _ =
                        event_sender.send(AppEvent::SaveSelected(self.saves[highlighted_row].id));
                } else if highlighted_row == self.saves.len() {
                    // View Pkroam database boxes
                    log::debug!("Request for pkroam database box data");
                } else {
                    // New save
                    log::debug!("Request to add new save");
                    let _ = event_sender.send(AppEvent::AddNewSave);
                }
            }
            KeyCode::Delete => {
                let highlighted_row = self.highlighted_row as usize;
                if (0..self.saves.len()).contains(&highlighted_row) {
                    log::debug!("Deleting save file");
                    let _ = event_sender
                        .send(AppEvent::RequestDeleteSave(self.saves[highlighted_row].id));
                }
            }
            _ => (),
        };
    }

    fn draw<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
                .split(frame.size());
            let mut text = self
                .saves
                .iter()
                .map(|save| Line::from(save.data.to_string()))
                .collect::<Vec<_>>();
            text.push(Line::from("<VIEW PKROAM BOXES ONLY>"));
            text.push(Line::from("<NEW SAVE>"));
            text[self.highlighted_row as usize].patch_style(
                Style::default()
                    .fg(ratatui::style::Color::DarkGray)
                    .bg(ratatui::style::Color::LightGreen),
            );

            let title = Block::default()
                .title(Span::styled(
                    "Select a save file to connect",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center);
            frame.render_widget(title, chunks[0]);

            let paragraph = Paragraph::new(text.clone())
                .block(Block::default().borders(Borders::ALL))
                .scroll((self.vertical_scroll as u16, 0));
            frame.render_widget(paragraph, chunks[1]);
            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                chunks[1],
                &mut self.vertical_scroll_state,
            );
        })?;

        Ok(())
    }

    pub fn update_saves(&mut self, saves: Vec<GameSave>) {
        if saves.len() != self.saves.len() {
            self.saves = saves;
            self.highlighted_row = 0;
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewSaveEntry {
    game_variants: Vec<Game>,
    save_path_in_progress: String,
    save_path: Option<PathBuf>,
    error_string: Option<String>,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    highlighted_row: usize,
}

impl NewSaveEntry {
    pub fn new() -> Self {
        Self {
            game_variants: Game::variants(),
            save_path_in_progress: String::new(),
            save_path: None,
            error_string: None,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            highlighted_row: 0,
        }
    }

    fn handle_key(&mut self, key_event: &KeyEvent, event_sender: &Sender<AppEvent>) {
        match (key_event.code, self.save_path.is_none()) {
            (KeyCode::Char(ch), true) => {
                self.save_path_in_progress.push(ch);
            }
            (KeyCode::Backspace, true) => {
                if self.save_path_in_progress.len() > 0 {
                    self.save_path_in_progress = self.save_path_in_progress
                        [..self.save_path_in_progress.len() - 1]
                        .to_owned();
                }
            }
            (KeyCode::Enter, true) => {
                if self.save_path_in_progress.len() > 0 {
                    let save_path = PathBuf::from(self.save_path_in_progress.as_str());
                    if save_path.exists() {
                        self.save_path = Some(save_path);
                        self.error_string = None;
                    } else {
                        let error_str = String::from("Save file path does not exist");
                        log::error!("{error_str}");
                        self.error_string = Some(error_str);
                    }
                }
            }
            (KeyCode::Backspace, false) => {
                self.save_path = None;
            }
            (KeyCode::Down, false) => {
                self.highlighted_row =
                    std::cmp::min(self.game_variants.len() - 1, self.highlighted_row + 1);
            }
            (KeyCode::Up, false) => {
                self.highlighted_row = std::cmp::max(0, self.highlighted_row - 1);
            }
            (KeyCode::Enter, false) => {
                let game_id = self.highlighted_row as u32;
                if let Some(game) = Game::from_u32(game_id) {
                    if let Ok(game_save) = SaveFile::new(&self.save_path.as_ref().unwrap()) {
                        let _ = event_sender.send(AppEvent::NewSaveCreated(
                            self.save_path.clone().unwrap(),
                            game,
                            game_save,
                        ));
                    } else {
                        let error_str = format!(
                            "Could not load save file from path: {}",
                            self.save_path.as_ref().unwrap().display()
                        );
                        log::error!("{error_str}");
                        self.error_string = Some(error_str);
                    }
                } else {
                    let error_str = format!("Invalid game id: {game_id}");
                    log::error!("{error_str}");
                    self.error_string = Some(error_str);
                }
            }
            _ => {}
        }
    }

    fn draw<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Percentage(50),
                ])
                .split(frame.size());

            let directions = Paragraph::new("Enter a path and select the game");
            frame.render_widget(directions, chunks[0]);

            let error = Paragraph::new(Span::styled(
                self.error_string.clone().unwrap_or(String::new()),
                Style::default().fg(ratatui::style::Color::Red),
            ));
            frame.render_widget(error, chunks[1]);

            let path_input_block = Paragraph::new(self.save_path_in_progress.clone())
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(path_input_block, chunks[2]);

            let mut options_text = self
                .game_variants
                .iter()
                .map(|game| Line::from(game.to_string()))
                .collect::<Vec<_>>();
            if self.save_path.is_some() {
                options_text[self.highlighted_row].patch_style(
                    Style::default()
                        .fg(ratatui::style::Color::DarkGray)
                        .bg(ratatui::style::Color::LightGreen),
                );
            }

            let game_options = Paragraph::new(options_text)
                .block(Block::default().borders(Borders::ALL))
                .scroll((self.vertical_scroll as u16, 0));
            frame.render_widget(game_options, chunks[3]);
            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                chunks[3],
                &mut self.vertical_scroll_state,
            );
        })?;
        Ok(())
    }
}
