//! File list component

use leptos::prelude::*;
use crate::store::FileStore;

#[component]
pub fn FileList() -> impl IntoView {
    let file_store = expect_context::<FileStore>();

    view! {
        <div class="space-y-4">
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
                                                    <div class="text-sm font-medium text-gray-900">{file_name}</div>
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
        </div>
    }
}
