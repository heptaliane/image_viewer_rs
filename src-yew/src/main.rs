use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;

use gloo::events::EventListener;
use wasm_logger;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::KeyboardEvent;
use yew::prelude::*;

mod key_action;

const DEFAULT_KEYMAP: [(&str, &str); 1] = [("ArrowRight", "NEXT_IMAGE")];

enum ImageViewMsg {
    OnKeyPress(KeyboardEvent),
}

struct ImageViewModel {
    source: RefCell<String>,
    keymap: HashMap<String, Box<&'static dyn Fn(RefCell<String>) -> ()>>,
    keybord_listener: Option<EventListener>,
}

impl Component for ImageViewModel {
    type Message = ImageViewMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            source: RefCell::new("".to_string()),
            keymap: key_action::create_keymap(
                DEFAULT_KEYMAP
                    .iter()
                    .map(|(action, key)| (action.to_string(), key.to_string()))
                    .collect(),
            ),
            keybord_listener: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::OnKeyPress(e) => match self.keymap.get(&e.key()) {
                Some(action) => action(self.source.clone()),
                _ => (),
            },
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container">
                <img
                    src={self.source.borrow().clone()}
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let document = gloo::utils::document();
            let onkeydown = ctx.link().callback(|e: KeyboardEvent| Self::Message::OnKeyPress(e));
            let listener = EventListener::new(&document, "keydown", move |e| {
                let event = e.dyn_ref::<KeyboardEvent>().unwrap_throw();
                onkeydown.emit(event.clone());
            });
            self.keybord_listener = Some(listener);
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<ImageViewModel>();
}
