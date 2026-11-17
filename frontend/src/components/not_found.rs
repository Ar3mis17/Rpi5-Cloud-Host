//! Not found page component

use leptos::prelude::*;
use leptos::ev;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NotFound() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="text-center">
                <h1 class="text-6xl font-bold text-gray-900">404</h1>
                <p class="mt-4 text-xl text-gray-600">Page not found</p>
                <button on:click=move |_| navigate("/dashboard", leptos_router::NavigateOptions::default()) class="mt-6 px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors">
                    Go to Dashboard
                </button>
            </div>
        </div>
    }
}
