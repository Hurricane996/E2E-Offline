use std::sync::Mutex;

use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

use crate::text_input::TextInput;

#[derive(Clone, PartialEq, Properties)]
pub struct SenderProps {
    pub shared_key: UseStateHandle<String>,
}
#[function_component(Sender)]

pub fn sender(props: &SenderProps) -> Html {
    let SenderProps { shared_key } = props.clone();
    let error_text = use_state(|| "".to_string());
    let sender_pubkey = use_state(|| "".to_string());
    let receiver_text = use_state(|| "".to_string());
    let connection_string_text = use_state(|| "".to_string());

    let on_receiver_text_change = {
        let receiver_text = receiver_text.clone();
        Callback::from(move |text| {
            receiver_text.set(text);
        })
    };

    static BUILDER_MUTEX: Mutex<Option<e2eoffline::E2EOfflineBuilder>> = Mutex::new(None);

    let create_sender = {
        let sender_pubkey = sender_pubkey.clone();
        let error_text = error_text.clone();
        Callback::from(move |_| {
            let mut builder = BUILDER_MUTEX.lock().unwrap();
            let sender = e2eoffline::E2EOfflineBuilder::new_sender();
            error_text.set("".to_string());

            sender_pubkey.set(
                sender
                    .get_pubkey_encoded()
                    .map_err(|e| error_text.set(e.to_string()))
                    .unwrap_throw(),
            );

            builder.replace(sender);
        })
    };

    let generate_connection_string = {
        let receiver_text = receiver_text.clone();
        let error_text = error_text.clone();
        let connection_string_text = connection_string_text.clone();
        let shared_key = shared_key.clone();
        Callback::from(move |_| {
            let mut builder = BUILDER_MUTEX.lock().unwrap();

            match builder.as_mut() {
                Some(builder) => {
                    error_text.set("".to_string());

                    builder
                        .set_other_public_key_encoded(&receiver_text)
                        .map_err(|_| error_text.set("Invalid reciever public key".to_string()))
                        .unwrap_throw();

                    shared_key.set(builder.get_shared_key().unwrap_throw());

                    connection_string_text.set(builder.send().unwrap_throw());
                }
                None => {
                    error_text.set("Need to generate a sender first".to_string());
                }
            }
        })
    };

    html! {
        <form onsubmit={Callback::from(|e: SubmitEvent| { e.prevent_default() })}>
        <h2>{ "Sender" }</h2>
        <p class="error">{(*error_text).clone()}</p>
        <div class="mb-3">
            <label for="sender_area_sender_pubkey" class="form-label">{"Sender Public Key" }</label>
            <input id="sender_area_sender_pubkey" class="form-control" disabled={true} value={(*sender_pubkey).clone()}/>
        </div>
        <button class="btn btn-primary" onclick={create_sender}>{ "Generate Sender" }</button>
        <div class="mb-3">
            <label for="sender_area_reciever_pubkey" class="form-label">{"Receiver Public Key" }</label>
            <TextInput class="form-control" id="sender_area_reciever_pubkey" value={(*receiver_text).clone()} on_change={on_receiver_text_change}/>
        </div>
        <div class="mb-3">
            <label for="sender_area_connection_string" class="form-label">{"Connection String"}</label>
            <input id="sender_area_connection_string" class="form-control" disabled={true} value = {(*connection_string_text).clone()}/>
        </div>
        <div class="mb-3">
            <label for="sender_area_shared_key" class="form-label">{"Shared Key"}</label>
            <input class="form-control" id="sender_area_shared_key" disabled={true} value = {(*shared_key).clone()}/>
        </div>
        <button class="btn btn-primary" onclick={generate_connection_string}>{ "Generate Shared Key" }</button>
        </form>
    }
}
