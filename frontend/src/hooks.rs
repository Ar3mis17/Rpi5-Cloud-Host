//! Custom hooks

use leptos::prelude::*;
use leptos_router::hooks::use_navigate as leptos_use_navigate;
use crate::store::AuthStore;

pub fn use_auth() -> AuthStore {
    expect_context::<AuthStore>()
}

pub fn use_navigate() -> impl Fn(&str, leptos_router::NavigateOptions) + 'static {
    let navigate = leptos_use_navigate();
    move |path, opts| navigate(path, opts)
}

pub fn provide_stores() {
    provide_context(crate::store::AuthStore::new());
    provide_context(crate::store::FileStore::new());
    provide_context(crate::store::UploadStore::new());
}
