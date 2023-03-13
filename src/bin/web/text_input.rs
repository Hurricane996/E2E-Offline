use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, HtmlTextAreaElement, InputEvent};
use yew::{function_component, html, Callback, Html, Properties};

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

#[function_component(TextAreaInput)]
pub fn text_area_input(props: &TextInputProps) -> Html {
    let TextInputProps {
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
