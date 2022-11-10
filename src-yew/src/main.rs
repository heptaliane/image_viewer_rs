use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;

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

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <img
                src={self.source.borrow().clone()}
                onkeydown={
                    ctx.link().callback(|e: KeyboardEvent| Self::Message::OnKeyPress(e))
                }
            />
        }
    }
}

fn main() {
    yew::start_app::<ImageViewModel>();
}
