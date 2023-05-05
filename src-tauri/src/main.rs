#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::{Manager, State};
use serde_json::Value;

mod image;
mod state;
mod path;

struct ViewerStateManager(Mutex<state::ViewerState>);

fn main() {
    tauri::Builder::default()
        .setup(move |app| match app.get_cli_matches() {
            Ok(matches) => match matches.args.get("filename").unwrap().value.clone() {
                Value::String(filename) => {
                    let state = state::ViewerState::new(&filename);
                    app.manage(ViewerStateManager(Mutex::new(state)));
                    Ok(())
                },
                _ => {
                    let msg = "Filename is required.";
                    println!("{:?}", msg);
                    Err(msg.into())
                }
            },
            Err(err) => {
                println!("{:?}", err.to_string());
                Err(err.into())
            }
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
                    Some(filename) => match image::try_get_source_image(filename) {
                        Ok(img) => return Ok(img),
                        _ => (),
                    },
                    _ => return Err("No valid image left".to_string()),
                }
            },
            Err(err) => Err(format!("{:?}", err)),
        },
        Err(err) => Err(format!("{:?}", err)),
    }
}
