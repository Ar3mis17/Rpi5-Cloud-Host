use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_logger;
use console_error_panic_hook;

mod app;
mod components;
mod api;
mod store;
mod hooks;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    mount_to_body(|| {
        view! {
            <Meta name="description" content="Ar3mis Cloud - Personal Cloud Storage"/>
            <Title text="Ar3mis Cloud"/>
            <Router>
                <App/>
            </Router>
        }
    })
}
