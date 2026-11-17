//! Login/Register page component

use leptos::prelude::*;
use leptos::ev;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use crate::store::AuthStore;
use crate::api::{login, register};

#[component]
pub fn Login() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let navigate = use_navigate();
    let (is_login, set_is_login) = signal(true);
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_loading.set(true);

        let username = username.get();
        let email = email.get();
        let password = password.get();
        let auth_store = auth_store.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            let result = if is_login.get() {
                login(username, password).await
            } else {
                register(username, email, password).await
            };

            match result {
                Ok(auth) => {
                    auth_store.login(auth.access_token, auth.refresh_token, auth.user);
                    navigate("/dashboard", Default::default());
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div>
                    <h2 class="text-center text-3xl font-extrabold text-gray-900">
                        {move || if is_login.get() { "Sign in to Ar3mis Cloud" } else { "Create your account" }}
                    </h2>
                </div>
                <form class="mt-8 space-y-6" on:submit=handle_submit>
                    {move || (!is_login.get()).then(|| view! {
                        <div class="rounded-md shadow-sm -space-y-px">
                            <div>
                                <label for="username" class="sr-only">Username</label>
                                <input
                                    id="username"
                                    name="username"
                                    type="text"
                                    required
                                    class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                    placeholder="Username"
                                    prop:value=username
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label for="email" class="sr-only">Email address</label>
                                <input
                                    id="email"
                                    name="email"
                                    type="email"
                                    required
                                    class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                    placeholder="Email address"
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    })}

                    <div class="rounded-md shadow-sm -space-y-px">
                        {move || is_login.get().then(|| view! {
                            <div>
                                <label for="username" class="sr-only">Username</label>
                                <input
                                    id="username"
                                    name="username"
                                    type="text"
                                    required
                                    class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                    placeholder="Username"
                                    prop:value=username
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                />
                            </div>
                        })}
                        <div>
                            <label for="password" class="sr-only">Password</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                required
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                placeholder="Password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div class="text-red-600 text-sm text-center">{e}</div>
                    })}

                    <div>
                        <button
                            type="submit"
                            disabled=loading
                            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            {move || if loading.get() { "Please wait..." } else if is_login.get() { "Sign in" } else { "Create account" }}
                        </button>
                    </div>
                </form>

                <div class="text-center">
                    <button
                        on:click=move |_| set_is_login.set(!is_login.get())
                        class="text-sm font-medium text-indigo-600 hover:text-indigo-500"
                    >
                        {move || if is_login.get() { "Need an account? Sign up" } else { "Already have an account? Sign in" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
