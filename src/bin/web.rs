use std::sync::Mutex;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

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

fn get_value_from_input_event(e: InputEvent) -> String {
    let event_target = e.dyn_into::<Event>().unwrap_throw().target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into::<HtmlInputElement>().unwrap_throw();
    target.value()
}

fn get_value_from_textarea_input_event(e: InputEvent) -> String {
    let event_target = e.dyn_into::<Event>().unwrap_throw().target().unwrap_throw();
    let target: HtmlTextAreaElement = event_target
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap_throw();
    target.value()
}

#[derive(Clone, PartialEq, Properties)]
struct ReceiverProps {
    shared_key: UseStateHandle<String>,
}

#[function_component(Receiver)]
fn receiver(props: &ReceiverProps) -> Html {
    let ReceiverProps { shared_key } = props.clone();

    let error_text = use_state(|| "".to_string());
    let receiver_pubkey_text = use_state(|| "".to_string());
    let sender_pubkey_text = use_state(|| "".to_string());
    let connection_string_text = use_state(|| "".to_string());

    let on_sender_pubkey_text_change = {
        let sender_pubkey_text = sender_pubkey_text.clone();
        Callback::from(move |text| {
            sender_pubkey_text.set(text);
        })
    };

    let on_connection_string_text_change = {
        let connection_string_text = connection_string_text.clone();
        Callback::from(move |text| {
            connection_string_text.set(text);
        })
    };

    static BUILDER_MUTEX: Mutex<Option<e2eoffline::E2EOfflineBuilder>> = Mutex::new(None);

    let generate_reciever = {
        let receiver_pubkey_text = receiver_pubkey_text.clone();
        let error_text = error_text.clone();

        Callback::from(move |_| {
            let mut builder = BUILDER_MUTEX.lock().unwrap();
            let reciever = e2eoffline::E2EOfflineBuilder::new_reciever();
            error_text.set("".to_string());

            receiver_pubkey_text.set(
                reciever
                    .get_pubkey_encoded()
                    .map_err(|e| error_text.set(e.to_string()))
                    .unwrap_throw(),
            );

            builder.replace(reciever);
        })
    };

    let generate_shared_key = {
        let sender_pubkey_text = sender_pubkey_text.clone();
        let error_text = error_text.clone();
        let connection_string_text = connection_string_text.clone();
        let shared_key = shared_key.clone();
        Callback::from(move |_| {
            let mut builder = BUILDER_MUTEX.lock().unwrap();

            match builder.as_mut() {
                Some(builder) => {
                    error_text.set("".to_string());

                    builder
                        .set_other_public_key_encoded(&sender_pubkey_text)
                        .map_err(|_| error_text.set("Invalid sender public key".to_string()))
                        .unwrap_throw();

                    builder
                        .recieve(&connection_string_text)
                        .map_err(|_| error_text.set("Invalid sender public key".to_string()))
                        .unwrap_throw();

                    shared_key.set(builder.get_shared_key().unwrap_throw());
                }
                None => {
                    error_text.set("Need to generate a receiver first".to_string());
                }
            }
        })
    };

    html! {
        <form onsubmit={Callback::from(|e: SubmitEvent| { e.prevent_default() })}>
            <h2>{ "Reciever" }</h2>
            <p class="error">{(*error_text).clone()}</p>
            <div class="mb-3">
                <label for="receiver_area_reciever_pubkey" class="form-label">{"Reciever Public Key"}</label>
                <input id="receiver_area_reciever_pubkey" class="form-control" disabled={true} value={(*receiver_pubkey_text).clone()}/>
            </div>
            <button class="btn btn-primary" onclick={generate_reciever}>{ "Generate Reciever" }</button>
            <div class="mb-3">
                <label for="reciever_area_sender_pubkey" class="form-label">{"Sender Public Key" }</label>
                <TextInput id="reciever_area_sender_pubkey" class="form-control" on_change={on_sender_pubkey_text_change} value={(*sender_pubkey_text).clone()}/>
            </div>
            <div class="mb-3">
                <label for="reciever_area_connection_string" class="form-label">{"Connection String" }</label>
                <TextInput id="reciever_area_connection_string" class="form-control" on_change={on_connection_string_text_change} value={(*connection_string_text).clone()}/>
            </div>
            <div class="mb-3">
                <label for="reciever_area_shared_key" class="form-label">{"Shared Key" }</label>
                <input value={(*shared_key).clone()} disabled={true} id="reciever_area_shared_key" class="form-control"/>
            </div>
            <button onclick={generate_shared_key} class="btn btn-primary">{ "Get Shared Key" }</button>
        </form>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct SenderProps {
    shared_key: UseStateHandle<String>,
}
#[function_component(Sender)]

fn sender(props: &SenderProps) -> Html {
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

#[derive(Clone, PartialEq, Properties)]
struct EncryptDecryptProps {
    shared_key: UseStateHandle<String>,
}
#[function_component(EncryptDecrypt)]
fn encrypt_decrypt(props: &EncryptDecryptProps) -> Html {
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

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone, PartialEq, Properties)]
pub struct TextInputProps {
    pub value: Option<String>,
    pub on_change: Callback<String>,
    pub class: Option<String>,
    pub id: Option<String>,
}

#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let TextInputProps {
        on_change,
        id,
        class,
        value,
    } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_input_event(input_event));
    });

    html! {
        <input type="text"  {oninput} {value} {class} {id}/>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct TextAreaInputProps {
    pub value: Option<String>,
    pub on_change: Callback<String>,
    pub class: Option<String>,
    pub id: Option<String>,
}

#[function_component(TextAreaInput)]
pub fn text_input(props: &TextAreaInputProps) -> Html {
    let TextAreaInputProps {
        on_change,
        id,
        class,
        value,
    } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_textarea_input_event(input_event));
    });

    html! {
        <textarea type="text"  {oninput} {value} {class} {id}/>
    }
}
