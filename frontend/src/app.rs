use leptos::*;
use leptos_router::*;
use crate::components::{Login, Dashboard, NotFound};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Routes>
            <Route path="/" view=Login/>
            <Route path="/login" view=Login/>
            <Route path="/dashboard" view=Dashboard/>
            <Route path="/dashboard/*any" view=Dashboard/>
            <Route path="/*any" view=NotFound/>
        </Routes>
    }
}
