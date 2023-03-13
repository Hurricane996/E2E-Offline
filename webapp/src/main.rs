use wasm_bindgen::JsValue;
use web_sys::console;
use yew::prelude::*;

mod encrypt_decrypt;
mod receiver;
mod sender;

mod text_input;

use encrypt_decrypt::EncryptDecrypt;
use receiver::Receiver;
use sender::Sender;

fn print_error_if_happened<T>(result: anyhow::Result<T>) {
    if let Err(e) = result {
        #[allow(unused_unsafe)]
        unsafe { console::error_1(&JsValue::from_str(&e.to_string())) };
    }
}

#[function_component(App)]
fn app() -> Html {
    let shared_key = use_state(|| "".to_string());

    html! {
        <div class="container">
            <div class="card-group">
                <div class="card">
                    <div class="card-body">
                        <Sender shared_key={shared_key.clone()} />
                    </div>
                </div>
                <div class="card">
                    <div class="card-body">
                        <Receiver shared_key={shared_key.clone()} />
                    </div>
                </div>
            </div>
            <div class="card-group">
                <div class="card">
                    <div class="card-body">
                        <EncryptDecrypt {shared_key}/>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
