#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use env_logger;
use serde_json::Value;
use std::{collections::HashSet, sync::Mutex};
use tauri::{Manager, State};

mod image;
mod path;
mod state;

static AVAILABLE_EXTENSIONS: [&str; 5] = ["bmp", "jpg", "jpeg", "png", "gif"];

struct ViewerStateManager(Mutex<state::ViewerState>);

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .setup(move |app| match app.get_cli_matches() {
            Ok(matches) => match matches.args.get("filename").unwrap().value.clone() {
                Value::String(filename) => {
                    let mut state = state::ViewerState::new(
                        &filename,
                        HashSet::from(AVAILABLE_EXTENSIONS.map(|s| s.to_string())),
                    );
                    match state.reload_files() {
                        Ok(_) => {
                            app.manage(ViewerStateManager(Mutex::new(state)));
                            Ok(())
                        }
                        Err(err) => Err(err.into()),
                    }
                }
                _ => Err("Filename is required".into()),
            },
            Err(err) => Err(err.into()),
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
            Ok(mut state) => loop {
                match state.move_cursor(n_moves) {
                    Ok(_) => (),
                    _ => return Err("No valid image left".to_string()),
                }
                match state.get() {
                    Ok(path) => match image::try_get_source_image(&path) {
                        Ok(img) => {
                            log::debug!("Current image: {:?}", path);
                            return Ok(img);
                        }
                        Err(err) => return Err(err),
                    },
                    _ => return Err("No valid image left".to_string()),
                }
            },
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}
