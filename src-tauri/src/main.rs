#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod image;

fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![move_image_offset])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn move_image_offset(moves: &str) -> Result<String, String> {
    let filename = "../sample.png".to_string();
    image::try_get_source_image(filename)
}
