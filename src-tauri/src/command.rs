use tauri::State;

use super::utils::{get_next_image, get_prev_image};
use super::ViewerStateManager;

#[tauri::command]
pub fn next_image(state_manager: State<ViewerStateManager>, moves: &str) -> Result<String, String> {
    match state_manager.0.lock() {
        Ok(mut state) => match moves.parse::<usize>() {
            Ok(n_moves) => get_next_image(&mut state, n_moves as i32),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}

#[tauri::command]
pub fn prev_image(state_manager: State<ViewerStateManager>, moves: &str) -> Result<String, String> {
    match state_manager.0.lock() {
        Ok(mut state) => match moves.parse::<usize>() {
            Ok(n_moves) => get_prev_image(&mut state, -(n_moves as i32)),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}

#[tauri::command]
pub fn next_directory(state_manager: State<ViewerStateManager>) -> Result<String, String> {
    match state_manager.0.lock() {
        Ok(mut state) => match state.next_directory() {
            Ok(_) => get_next_image(&mut state, 0),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}

#[tauri::command]
pub fn prev_directory(state_manager: State<ViewerStateManager>) -> Result<String, String> {
    match state_manager.0.lock() {
        Ok(mut state) => match state.prev_directory() {
            Ok(_) => get_prev_image(&mut state, 0),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}
