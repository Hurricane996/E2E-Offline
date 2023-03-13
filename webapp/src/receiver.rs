use web_sys::SubmitEvent;
use yew::prelude::*;

use crate::{print_error_if_happened, text_input::TextInput};

#[derive(Clone, PartialEq, Properties)]
pub struct ReceiverProps {
    pub shared_key: UseStateHandle<String>,
}

#[function_component(Receiver)]
pub fn receiver(props: &ReceiverProps) -> Html {
    let ReceiverProps { shared_key } = props.clone();

    let error_text = use_state(|| "".to_string());
    let receiver_pubkey_text = use_state(|| "".to_string());
    let sender_pubkey_text = use_state(|| "".to_string());
    let connection_string_text = use_state(|| "".to_string());

    let is_doing_work = use_state(|| false);

    let builder = use_mut_ref(|| None);

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

    let generate_reciever = {
        let receiver_pubkey_text = receiver_pubkey_text.clone();
        let error_text = error_text.clone();
        let builder = builder.clone();
        let is_doing_work = is_doing_work.clone();

        Callback::from(move |_| {
            is_doing_work.set(true);
            let r = (|| -> anyhow::Result<()> {
                let reciever = e2eoffline::E2EOfflineBuilder::new_reciever();
                error_text.set("".to_string());

                receiver_pubkey_text.set(reciever.get_pubkey_encoded().map_err(|e| {
                    error_text.set(e.to_string());
                    e
                })?);

                builder.replace(Some(reciever));
                Ok(())
            })();
            is_doing_work.set(false);

            print_error_if_happened(r);
        })
    };

    let generate_shared_key = {
        let sender_pubkey_text = sender_pubkey_text.clone();
        let error_text = error_text.clone();
        let connection_string_text = connection_string_text.clone();
        let shared_key = shared_key.clone();
        let is_doing_work = is_doing_work.clone();
        Callback::from(move |_| {
            is_doing_work.set(true);
            let r = (|| -> anyhow::Result<()> {
                match (*builder).borrow_mut().as_mut() {
                    Some(builder) => {
                        error_text.set("".to_string());

                        builder
                            .set_other_public_key_encoded(&sender_pubkey_text)
                            .map_err(|e| {
                                error_text.set("Invalid sender public key".to_string());
                                e
                            })?;

                        builder.recieve(&connection_string_text).map_err(|e| {
                            error_text.set("Invalid connection string text".to_string());
                            e
                        })?;

                        shared_key.set(builder.get_shared_key()?);
                    }
                    None => {
                        error_text.set("Need to generate a receiver first".to_string());
                    }
                }
                Ok(())
            })();
            is_doing_work.set(false);
            print_error_if_happened(r);
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
            <button class="btn btn-primary" onclick={generate_reciever} disabled={*is_doing_work}>{ "Generate Reciever" }</button>
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
            <button onclick={generate_shared_key} class="btn btn-primary" disabled={*is_doing_work}>{ "Get Shared Key" }</button>
        </form>
    }
}
