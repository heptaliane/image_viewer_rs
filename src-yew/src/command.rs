use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::Callback;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = nextImage, catch)]
    async fn next_image(moves: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = prevImage, catch)]
    async fn prev_image(moves: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = nextDirectory, catch)]
    async fn next_directory(moves: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = prevDirectory, catch)]
    async fn prev_directory(moves: &str) -> Result<JsValue, JsValue>;
}

pub fn fetch_current_image_source(handler: Callback<String>) {
    spawn_local(async move {
        match next_image("0").await {
            Ok(data) => {
                if let Some(src) = data.as_string() {
                    handler.emit(src);
                }
            }
            Err(err) => log::error!("{:?}", err),
        }
    });
}

pub fn fetch_next_image_source(handler: Callback<String>) {
    spawn_local(async move {
        match next_image("1").await {
            Ok(data) => {
                if let Some(src) = data.as_string() {
                    handler.emit(src);
                }
            }
            Err(err) => log::error!("{:?}", err),
        }
    });
}

pub fn fetch_prev_image_source(handler: Callback<String>) {
    spawn_local(async move {
        match prev_image("1").await {
            Ok(data) => {
                if let Some(src) = data.as_string() {
                    handler.emit(src);
                }
            }
            Err(err) => log::error!("{:?}", err),
        }
    });
}
