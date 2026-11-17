use leptos::prelude::*;
use leptos::children::Children;
use leptos_router::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            {children()}
        </div>
    }
}
