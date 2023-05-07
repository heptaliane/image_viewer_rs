#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use env_logger;
use serde_json::Value;
use std::{collections::HashSet, sync::Mutex};
use tauri::Manager;

mod command;
mod image;
mod path;
mod state;
mod utils;

static AVAILABLE_EXTENSIONS: [&str; 5] = ["bmp", "jpg", "jpeg", "png", "gif"];

pub struct ViewerStateManager(Mutex<state::ViewerState>);

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
        .invoke_handler(tauri::generate_handler![
            command::next_image,
            command::prev_image,
            command::next_directory,
            command::prev_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
