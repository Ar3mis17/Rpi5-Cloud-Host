//! Custom hooks

use leptos::*;
use leptos_router::*;
use crate::store::AuthStore;

pub fn use_auth() -> AuthStore {
    expect_context::<AuthStore>()
}

pub fn use_navigate() -> impl Fn(&str, Default) + 'static {
    let navigate = leptos_router::use_navigate();
    move |path, opts| navigate(path, opts)
}

pub fn provide_stores() {
    provide_context(crate::store::AuthStore::new());
    provide_context(crate::store::FileStore::new());
    provide_context(crate::store::UploadStore::new());
}
