#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::{Manager, State};

mod image;
mod state;

struct ViewerStateManager(Mutex<state::ViewerState>);

fn main() {
    let filename = "../sample.png".to_string();
    tauri::Builder::default()
        .setup(move |app| {
            let state = state::ViewerState::new(&filename);
            app.manage(ViewerStateManager(Mutex::new(state)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![move_image_offset])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn move_image_offset(
    state_manager: State<ViewerStateManager>,
    moves: &str,
) -> Result<String, String> {
    match moves.parse::<i32>() {
        Ok(n_moves) => match state_manager.0.lock() {
            Ok(mut state) => {
                state.move_cursor(n_moves);
                match state.get() {
                    Some(filename) => image::try_get_source_image(filename),
                    None => Err(format!("Target not found")),
                }
            }
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}
