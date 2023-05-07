use std::boxed::Box;
use std::collections::HashMap;

use yew::prelude::*;

use super::command;

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

const KEY_ACTION_MAP: [(KeyAction, &dyn Fn(Callback<String>) -> ()); 2] = [
    (KeyAction::NextImage, &command::fetch_next_image_source),
    (KeyAction::PrevImage, &command::fetch_prev_image_source),
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
