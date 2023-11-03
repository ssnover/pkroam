use pkroam::save::SaveFile;

use crate::{
    database::DbConn,
    types::{Game, GameSave, GameSaveData},
};
use std::sync::{
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
use std::{ops::DerefMut, path::PathBuf};

const BACKEND_SLEEP_TIME_MILLIS: u64 = 30;

/// This holds an application state which represents the UI state and holds
/// the data necessary for rendering the UI, filled in by requests to the backend
/// context.
#[derive(Clone, Debug)]
pub enum AppState {
    /// Show a list of saves from the database for selection and allow user to add new save
    SaveSelection(Vec<GameSave>),
    /// Show a menu to enter data for new save
    NewSave,
    ConnectedSaveEdit(GameSaveData),
    // PkRoamBoxEdit,
}

impl AppState {
    fn name(&self) -> &'static str {
        match self {
            AppState::SaveSelection(_) => "save_selection",
            AppState::NewSave => "new_save",
            AppState::ConnectedSaveEdit(_) => "connected_save_edit",
        }
    }
}

pub enum AppEvent {
    AddNewSave,
    SaveSelected(u64),
    RequestDeleteSave(u64),
    NewSaveCreated(PathBuf, Game, SaveFile),
}

impl AppEvent {
    fn name(&self) -> &'static str {
        match self {
            AppEvent::AddNewSave => "new_save",
            AppEvent::SaveSelected(_) => "save_selected",
            AppEvent::RequestDeleteSave(_) => "request_delete_save",
            AppEvent::NewSaveCreated(_, _, _) => "new_save_created",
        }
    }
}

pub struct BackendHandle {
    terminate_sender: Sender<()>,
    thread_handle: std::thread::JoinHandle<()>,
}

impl BackendHandle {
    pub fn quit(self) {
        match self.terminate_sender.send(()) {
            Ok(()) => log::info!("Successfully signalled backend to exit"),
            Err(_) => log::error!("Backend context already closed, exiting anyway"),
        }

        match self.thread_handle.join() {
            Ok(()) => log::info!("Exiting"),
            Err(err) => log::error!("Backend context panicked: {err:?}"),
        }
    }
}

pub fn start_app_backend(
    db_handle: DbConn,
) -> rusqlite::Result<(BackendHandle, Sender<AppEvent>, Arc<Mutex<AppState>>)> {
    let (terminate_tx, terminate_rx) = channel();
    let (event_tx, event_rx) = channel();

    let game_saves = db_handle.get_saves()?;
    let app_state = Arc::new(Mutex::new(AppState::SaveSelection(game_saves)));

    let frontend_app_state = app_state.clone();
    let backend_context = std::thread::spawn(|| {
        backend_context(terminate_rx, frontend_app_state, db_handle, event_rx)
    });

    Ok((
        BackendHandle {
            terminate_sender: terminate_tx,
            thread_handle: backend_context,
        },
        event_tx,
        app_state,
    ))
}

fn backend_context(
    terminate_rx: Receiver<()>,
    app_state: Arc<Mutex<AppState>>,
    db_handle: DbConn,
    event_rx: Receiver<AppEvent>,
) -> () {
    loop {
        let start_loop = std::time::Instant::now();

        match terminate_rx.try_recv() {
            Ok(()) => {
                log::info!("Backend received request to exit");
                break;
            }
            Err(TryRecvError::Disconnected) => {
                log::error!("Frontend disconnected termination channel");
                break;
            }
            Err(TryRecvError::Empty) => (),
        }

        match event_rx.try_recv() {
            Ok(event) => {
                log::info!("Backend received event: {}", event.name());
                handle_event(&app_state, &db_handle, event);
            }
            Err(TryRecvError::Disconnected) => {
                log::error!("Frontend disconnected event channel");
                break;
            }
            Err(TryRecvError::Empty) => (),
        }

        std::thread::sleep(
            std::time::Duration::from_millis(BACKEND_SLEEP_TIME_MILLIS) - start_loop.elapsed(),
        );
    }
}

fn handle_event(app_state: &Arc<Mutex<AppState>>, db_handle: &DbConn, event: AppEvent) {
    let mut current_state = app_state.lock().unwrap();
    match (&*current_state, event) {
        (AppState::SaveSelection(_), AppEvent::AddNewSave) => {
            log::info!("Backend received request to add new save");
            *current_state = AppState::NewSave;
        }
        (AppState::SaveSelection(saves), AppEvent::RequestDeleteSave(save_id)) => {
            if let Err(err) = db_handle.delete_save(save_id) {
                log::error!("Unable to delete save data: {err:?}");
            } else {
                let saves = saves
                    .iter()
                    .filter(|save| save.id == save_id)
                    .cloned()
                    .collect::<Vec<_>>();
                *current_state = AppState::SaveSelection(saves);
            }
        }
        (AppState::SaveSelection(saves), AppEvent::SaveSelected(save_id)) => {
            *current_state = AppState::ConnectedSaveEdit(
                saves
                    .iter()
                    .find(|save| save.id == save_id)
                    .unwrap()
                    .data
                    .clone(),
            )
        }
        (AppState::NewSave, AppEvent::NewSaveCreated(save_path, game, save)) => {
            let trainer_info = save.get_trainer_info();

            let game_save_data = GameSaveData::new(
                game,
                &trainer_info.player_name,
                trainer_info.id.public_id.into(),
                trainer_info.id.secret_id.into(),
                save_path,
            );
            if let Ok(()) = db_handle.add_new_save(&game_save_data) {
                *current_state = AppState::ConnectedSaveEdit(game_save_data);
            } else {
                log::error!("Unable to add save data: {game_save_data:?}");
            }
        }
        (state, event) => {
            log::error!("Unhandled event {} in state {}", event.name(), state.name());
        }
    }
}
