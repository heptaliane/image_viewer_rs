use std::boxed::Box;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = moveImageOffset, catch)]
    pub async fn move_image_offset(moves: &str) -> Result<JsValue, JsValue>;
}

fn fetch_image_source(handler: Callback<String>, moves: i32) {
    spawn_local(async move {
        match move_image_offset(moves.to_string().as_str()).await {
            Ok(data) => {
                if let Some(src) = data.as_string() {
                    handler.emit(src);
                }
            }
            Err(e) => log::error!("{:?}", e),
        }
    });
}

pub fn fetch_current_image_source(handler: Callback<String>) {
    fetch_image_source(handler, 0);
}

fn fetch_next_image_source(handler: Callback<String>) {
    fetch_image_source(handler, 1);
}

fn fetch_prev_image_source(handler: Callback<String>) {
    fetch_image_source(handler, -1);
}

enum KeyAction {
    NextImage,
    PrevImage,
}

impl KeyAction {
    fn as_string(self) -> String {
        match self {
            KeyAction::NextImage => "NEXT_IMAGE",
            KeyAction::PrevImage => "PREV_IMAGE",
        }
        .to_string()
    }
}

const KEY_ACTION_MAP: [(KeyAction, &dyn Fn(Callback<String>) -> ()); 2] =
    [
        (KeyAction::NextImage, &fetch_next_image_source),
        (KeyAction::PrevImage, &fetch_prev_image_source),
    ];

pub fn create_keymap(
    keyset: HashMap<String, String>,
) -> HashMap<String, Box<&'static dyn Fn(Callback<String>) -> ()>> {
    let actions: HashMap<String, Box<&dyn Fn(Callback<String>) -> ()>> =
        HashMap::from_iter(KEY_ACTION_MAP.map(|(k, func)| (k.as_string(), Box::new(func))));

    keyset.iter().fold(
        HashMap::<String, Box<&dyn Fn(Callback<String>) -> ()>>::new(),
        |mut map, (key, action)| {
            if let Some(func) = actions.get(action) {
                map.insert(key.clone(), func.clone());
            }
            map
        },
    )
}
