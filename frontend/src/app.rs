use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, Redirect};
use crate::components::{Login, Dashboard, NotFound};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path=path!("/") view=Login/>
                <Route path=path!("/login") view=Login/>
                <Route path=path!("/dashboard") view=Dashboard/>
                <Route path=path!("/dashboard/*any") view=Dashboard/>
                <Route path=path!("/*any") view=NotFound/>
            </Routes>
        </Router>
    }
}
