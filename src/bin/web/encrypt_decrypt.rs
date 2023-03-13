use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

use crate::text_input::{TextAreaInput, TextInput};

#[derive(Clone, PartialEq, Properties)]
pub struct EncryptDecryptProps {
    pub shared_key: UseStateHandle<String>,
}
#[function_component(EncryptDecrypt)]
pub fn encrypt_decrypt(props: &EncryptDecryptProps) -> Html {
    let EncryptDecryptProps { shared_key } = props.clone();
    // it's fine that this isn't a use_state because we don't need changes to it to trigger

    let plain_text = use_state(|| "".to_string());
    let cipher_text = use_state(|| "".to_string());

    let error_text = use_state(|| "".to_string());

    let on_key_change = {
        let shared_key = shared_key.clone();
        Callback::from(move |text| {
            shared_key.set(text);
        })
    };

    let on_plain_text_change = {
        let plain_text = plain_text.clone();
        Callback::from(move |text| {
            plain_text.set(text);
        })
    };

    let on_cipher_text_change = {
        let cipher_text = cipher_text.clone();
        Callback::from(move |text| {
            cipher_text.set(text);
        })
    };

    let encrypt = {
        let plain_text = plain_text.clone();
        let cipher_text = cipher_text.clone();
        let error_text = error_text.clone();
        let shared_key = shared_key.clone();

        Callback::from(move |_| {
            let mut aes = e2eoffline::E2EOffline::from_key_base64(&shared_key)
                .map_err(|_| error_text.set("Bad AES Key".to_string()))
                .unwrap_throw();

            error_text.set("".to_string());
            cipher_text.set(aes.encrypt(&plain_text).unwrap_throw());
        })
    };

    let decrypt = {
        let plain_text = plain_text.clone();
        let cipher_text = cipher_text.clone();
        let error_text = error_text.clone();
        let shared_key = shared_key.clone();
        Callback::from(move |_| {
            error_text.set("".to_string());
            let mut aes = e2eoffline::E2EOffline::from_key_base64(&shared_key)
                .map_err(|_| error_text.set("Bad AES Key".to_string()))
                .unwrap_throw();

            let text = aes
                .decrypt(&cipher_text)
                .map_err(|_| error_text.set("Bad Ciphertext".to_string()))
                .unwrap_throw();

            plain_text.set(text)
        })
    };

    html! {
        <form onsubmit={Callback::from(|e: SubmitEvent| { e.prevent_default() })}>
            <h2>{ "Encrypt/Decrypt"}</h2>
            <p class="error">{(*error_text).clone()}</p>
            <div class="mb-3">
                <label for="ed_area_key">{"Key"}</label>
                <TextInput id="ed_area_key" class={"form-control"} on_change={on_key_change} value={(*shared_key).clone()}/>
            </div>
            <div class="mb-3">
                <label for="plaintext">{"Plaintext"}</label>
                <TextAreaInput class="form-control" id="ciphertext" on_change={on_plain_text_change} value={(*plain_text).clone()}/>
            </div>
            <button onclick={encrypt} class="btn btn-primary">{"Encrypt"}</button>
            <div class="mb-3">
                <label for="ciphertext">{"Ciphertext"}</label>
                <TextAreaInput class="form-control" id="ciphertext" on_change={on_cipher_text_change} value={(*cipher_text).clone()}/>
            </div>
            <button onclick={decrypt} class="btn btn-primary">{"Decrypt"}</button><br/>
        </form>
    }
}
