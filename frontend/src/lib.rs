//! Frontend library entry point

mod api;
mod app;
mod components;
mod hooks;
mod store;
mod utils;

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

use crate::app::App;

pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    mount_to_body(|| view! { <App/> });
}
