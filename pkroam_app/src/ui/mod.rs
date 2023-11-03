use crate::app::{AppEvent, AppState};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use self::states::{NewSaveEntry, SaveSelection, UiState};

mod states;

pub struct UiManager<'a, B: Backend> {
    app_state: Arc<Mutex<AppState>>,
    event_sender: Sender<AppEvent>,
    terminal: &'a mut Terminal<B>,
    last_key_event: Option<Event>,
    ui_state: UiState,
}

pub fn run_app_ui(
    app_state: Arc<Mutex<AppState>>,
    event_sender: Sender<AppEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut ui_manager = UiManager {
        app_state: app_state.clone(),
        event_sender,
        terminal: &mut terminal,
        last_key_event: None,
        ui_state: initialize_ui_state_from_app_state(&*app_state.lock().unwrap()).unwrap(),
    };
    let _ = ui_manager.run_context();

    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

impl<'a, B> UiManager<'a, B>
where
    B: Backend,
{
    fn run_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.last_key_event = poll_keyboard()?;
            if let Some(key_event) = self.last_key_event.as_ref() {
                if check_for_exit_request(key_event) {
                    break;
                }
            }

            self.update_ui_state();
            if let Some(Event::Key(key_event)) = self.last_key_event.as_ref() {
                self.ui_state.handle_key(key_event, &self.event_sender)
            }
            self.ui_state.draw(self.terminal)?;
        }

        Ok(())
    }

    fn update_ui_state(&mut self) {
        let current_state = self.app_state.lock().unwrap().clone();
        match (current_state, &self.ui_state) {
            (AppState::SaveSelection(saves), UiState::SaveSelection(_)) => {
                let UiState::SaveSelection(data) = &mut self.ui_state else {
                    unreachable!()
                };
                data.update_saves(saves);
            }
            (AppState::SaveSelection(saves), _) => {
                self.ui_state = UiState::SaveSelection(SaveSelection::new(saves));
            }
            (AppState::NewSave, UiState::NewSaveEntry(_)) => {}
            (AppState::NewSave, _) => self.ui_state = UiState::NewSaveEntry(NewSaveEntry::new()),
            _ => unimplemented!(),
        }
    }
}

fn poll_keyboard() -> Result<Option<Event>, Box<dyn std::error::Error>> {
    if let Ok(false) = crossterm::event::poll(std::time::Duration::from_millis(30)) {
        return Ok(None);
    }
    let event = crossterm::event::read()?;
    if matches!(event, Event::Key(_)) {
        Ok(Some(event))
    } else {
        Ok(None)
    }
}

fn check_for_exit_request(key_event: &Event) -> bool {
    if let Event::Key(key_event) = key_event {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn initialize_ui_state_from_app_state(app_state: &AppState) -> Option<UiState> {
    if let AppState::SaveSelection(saves) = app_state {
        Some(UiState::SaveSelection(SaveSelection::new(saves.clone())))
    } else {
        None
    }
}
