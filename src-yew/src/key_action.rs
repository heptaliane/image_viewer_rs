use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = moveImageOffset, catch)]
    pub async fn move_image_offset(moves: &str) -> Result<JsValue, JsValue>;
}

fn fetch_next_image_source(source: RefCell<String>) {
    spawn_local(async move {
        match move_image_offset("1").await {
            Ok(data) => {
                if let Some(src) = data.as_string() {
                    log::info!("{}", src);
                    source.replace(src);
                }
            }
            Err(e) => log::error!("{:?}", e),
        }
    });
}

enum KeyAction {
    NextImage,
}

impl KeyAction {
    fn as_string(self) -> String {
        match self {
            KeyAction::NextImage => "NEXT_IMAGE",
        }
        .to_string()
    }
}

const KEY_ACTION_MAP: [(KeyAction, &dyn Fn(RefCell<String>) -> ()); 1] =
    [(KeyAction::NextImage, &fetch_next_image_source)];

pub fn create_keymap(
    keyset: HashMap<String, String>,
) -> HashMap<String, Box<&'static dyn Fn(RefCell<String>) -> ()>> {
    let actions: HashMap<String, Box<&dyn Fn(RefCell<String>) -> ()>> =
        HashMap::from_iter(KEY_ACTION_MAP.map(|(k, func)| (k.as_string(), Box::new(func))));

    keyset.iter().fold(
        HashMap::<String, Box<&dyn Fn(RefCell<String>) -> ()>>::new(),
        |mut map, (key, action)| {
            if let Some(func) = actions.get(action) {
                map.insert(key.clone(), func.clone());
            }
            map
        },
    )
}
