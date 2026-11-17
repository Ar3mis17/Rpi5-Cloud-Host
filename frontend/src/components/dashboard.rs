//! Dashboard page component

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use js_sys::wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlInputElement;
use shared::{CreateFolderRequest, UploadInitiateRequest};
use crate::store::{AuthStore, FileStore, UploadStore};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let file_store = expect_context::<FileStore>();
    let upload_store = expect_context::<UploadStore>();
    let navigate = use_navigate();

    let load_files = move || {
        spawn_local(async move {
            file_store.set_loading(true);
            match crate::api::list_files(file_store.current_folder.get(), file_store.pagination.get().page, file_store.page_size.get()).await {
                Ok(response) => file_store.set_files(response),
                Err(e) => {
                    file_store.set_loading(false);
                    leptos::logging::error!("Failed to load files: {}", e);
                }
            }
        });
    };

    let handle_folder_click = move |folder_id: i64| {
        file_store.set_current_folder(Some(folder_id));
        load_files();
    };

    let handle_back_click = move |_| {
        let breadcrumbs = file_store.breadcrumbs.get();
        if breadcrumbs.len() > 1 {
            let parent_id = breadcrumbs[breadcrumbs.len() - 2].id;
            file_store.set_current_folder(parent_id);
            load_files();
        } else {
            file_store.set_current_folder(None);
            load_files();
        }
    };

    let handle_new_folder = move |_| {
        let name = prompt("Enter folder name:").unwrap_or_default();
        if !name.is_empty() {
            spawn_local(async move {
                match crate::api::create_folder(CreateFolderRequest { name, parent_id: file_store.current_folder.get() }).await {
                    Ok(_) => load_files(),
                    Err(e) => leptos::logging::error!("Failed to create folder: {}", e),
                }
            });
        }
    };

    let handle_upload = move |_| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let input = document.create_element("input").unwrap();
        input.set_attribute("type", "file").unwrap();
        input.set_attribute("multiple", "").unwrap();
        let input: web_sys::HtmlInputElement = input.dyn_into().unwrap();
        let upload_store = upload_store.clone();
        let file_store = file_store.clone();
        let auth_store = auth_store.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let file_clone = file.clone();
                        let file_store = file_store.clone();
                        let upload_store = upload_store.clone();
                        let auth_store = auth_store.clone();
                        spawn_local(async move {
                            match crate::api::initiate_upload(UploadInitiateRequest {
                                filename: file_clone.name(),
                                size_bytes: file_clone.size() as i64,
                                mime_type: Some(file_clone.type_()),
                                parent_id: file_store.current_folder.get(),
                            }).await {
                                Ok(response) => {
                                    upload_store.add_upload(response.upload_id, file_clone, response.total_chunks);
                                    upload_chunks(response.upload_id, file_clone, response.total_chunks, file_store, upload_store, auth_store).await;
                                }
                                Err(e) => leptos::logging::error!("Failed to initiate upload: {}", e),
                            }
                        });
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
        input.click();
    };

    async fn upload_chunks(upload_id: uuid::Uuid, file: web_sys::File, total_chunks: i64, file_store: FileStore, upload_store: UploadStore, _auth_store: AuthStore) {
        let chunk_size = 1024 * 1024;
        let file_size = file.size() as usize;
        let total_chunks = ((file_size + chunk_size - 1) / chunk_size) as i32;

        for chunk_num in 1..=total_chunks {
            let start = (chunk_num - 1) as usize * chunk_size;
            let end = std::cmp::min(start + chunk_size, file_size);
            let chunk_blob = file.slice_with_f64_and_f64(start as f64, end as f64).unwrap();
            let chunk_array = js_sys::Uint8Array::new(&chunk_blob);
            let mut chunk_data = vec![0u8; chunk_array.length() as usize];
            chunk_array.copy_to(&mut chunk_data);

            match crate::api::upload_chunk(upload_id, chunk_num, chunk_data).await {
                Ok(_) => {
                    upload_store.update_progress(upload_id, chunk_num as i64, total_chunks as i64);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to upload chunk {}: {}", chunk_num, e);
                    return;
                }
            }
        }

        match crate::api::complete_upload(upload_id).await {
            Ok(file_entry) => {
                upload_store.remove_upload(upload_id);
                file_store.add_file(file_entry);
            }
            Err(e) => leptos::logging::error!("Failed to complete upload: {}", e),
        }
    };

    view! {
        <div class="min-h-screen bg-gray-50">
            <header class="bg-white shadow-sm border-b">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex justify-between items-center h-16">
                        <div class="flex items-center">
                            <h1 class="text-xl font-bold text-gray-900">Ar3mis Cloud</h1>
                        </div>
                        <div class="flex items-center gap-4">
                            <div class="flex items-center gap-2">
                                <span class="text-sm text-gray-600">{move || auth_store.user.get().map(|u| u.username).unwrap_or_default()}</span>
                            </div>
                            <button on:click=move |_| {
                                auth_store.logout();
                                navigate("/login", Default::default());
                            } class="text-sm text-gray-600 hover:text-gray-900">Sign out</button>
                        </div>
                    </div>
                </div>
            </header>

            <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <div class="flex justify-between items-center mb-6">
                    <div>
                        <nav class="flex" aria-label="Breadcrumb">
                            <ol class="flex items-center space-x-2">
                                <li>
                                    <button on:click=handle_back_click class="text-sm font-medium text-indigo-600 hover:text-indigo-500 flex items-center gap-1">
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>
                                        Root
                                    </button>
                                </li>
                                {move || file_store.breadcrumbs.get().into_iter().skip(1).map(|crumb| view! {
                                    <li>
                                        <svg class="w-4 h-4 text-gray-400" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/></svg>
                                        <span class="ml-2 text-sm font-medium text-gray-500">{crumb.name.clone()}</span>
                                    </li>
                                }).collect::<Vec<_>>()}
                            </ol>
                        </nav>
                    </div>
                    <div class="flex gap-2">
                        <button on:click=handle_new_folder class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 text-sm font-medium flex items-center gap-2">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                            New Folder
                        </button>
                        <button on:click=handle_upload class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm font-medium flex items-center gap-2">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                            Upload
                        </button>
                    </div>
                </div>

                {move || file_store.is_loading.get().then(|| view! {
                    <div class="flex justify-center py-12">
                        <svg class="animate-spin h-8 w-8 text-indigo-600" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/></svg>
                    </div>
                })}

                {move || (!file_store.is_loading.get() && file_store.files.get().is_empty()).then(|| view! {
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900">No files yet</h3>
                        <p class="mt-1 text-sm text-gray-500">Get started by creating a folder or uploading files.</p>
                        <div class="mt-6">
                            <button on:click=handle_new_folder class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 text-sm font-medium flex items-center gap-2 mx-auto">
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                                Create Folder
                            </button>
                        </div>
                    </div>
                })}

                {move || (!file_store.is_loading.get() && !file_store.files.get().is_empty()).then(|| view! {
                    <div class="overflow-hidden bg-white shadow-sm rounded-lg border">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Size</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Modified</th>
                                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {move || file_store.files.get().into_iter().map(|file| {
                                    let file_id = file.id;
                                    let file_name = file.name.clone();
                                    let is_folder = file.is_folder();
                                    let file_size = file.size_bytes;
                                    let updated_at = file.updated_at.format("%Y-%m-%d %H:%M").to_string();
                                    view! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <div class="flex items-center">
                                                    {move || if is_folder {
                                                        view! {
                                                            <svg class="h-8 w-8 text-yellow-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                                                        }
                                                    } else {
                                                        view! {
                                                            <svg class="h-8 w-8 text-gray-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"/></svg>
                                                        }
                                                    }}
                                                    <div class="ml-4">
                                                        <div class="text-sm font-medium text-gray-900">
                                                            {move || {
                                                                let file_id = file_id;
                                                                let handle_click = handle_folder_click.clone();
                                                                view! {
                                                                    <span
                                                                        class=if is_folder { "text-indigo-600 hover:text-indigo-900 cursor-pointer" } else { "" }
                                                                        on:click=move |_| {
                                                                            if is_folder {
                                                                                handle_click(file_id);
                                                                            }
                                                                        }
                                                                    >
                                                                        {file_name.clone()}
                                                                    </span>
                                                                }
                                                            }}
                                                        </div>
                                                    </div>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                {move || if is_folder { "\u{2014}".to_string() } else { crate::utils::format_bytes(file_size) }}
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                {updated_at}
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                <div class="flex items-center justify-end gap-2">
                                                    {move || (!is_folder).then(|| view! {
                                                            <a href=format!("/api/v1/files/{}/download", file_id) class="text-indigo-600 hover:text-indigo-900">Download</a>
                                                        })}
                                                    <button class="text-red-600 hover:text-red-900">Delete</button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                })}

                {move || (!file_store.is_loading.get() && file_store.pagination.get().total_pages > 1).then(|| view! {
                    <div class="mt-6 flex justify-center">
                        <nav class="flex items-center gap-2">
                            <button disabled=move || file_store.pagination.get().page <= 1 on:click=move |_| { file_store.pagination.update(|p| p.page -= 1); load_files(); } class="px-3 py-2 border rounded-lg disabled:opacity-50">Previous</button>
                            <span class="px-3 py-2">Page {move || file_store.pagination.get().page} of {move || file_store.pagination.get().total_pages}</span>
                            <button disabled=move || file_store.pagination.get().page >= file_store.pagination.get().total_pages on:click=move |_| { file_store.pagination.update(|p| p.page += 1); load_files(); } class="px-3 py-2 border rounded-lg disabled:opacity-50">Next</button>
                        </nav>
                    </div>
                })}
            </main>
        </div>
    }
}
