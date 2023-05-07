use tauri::State;

use super::utils::{get_next_image, get_prev_image};
use super::ViewerStateManager;

#[tauri::command]
pub fn move_image_offset(
    state_manager: State<ViewerStateManager>,
    moves: &str,
) -> Result<String, String> {
    match state_manager.0.lock() {
        Ok(mut state) => match moves.parse::<i32>() {
            Ok(n_moves) if n_moves >= 0 => get_next_image(&mut state, n_moves),
            Ok(n_moves) => get_prev_image(&mut state, n_moves),
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}
