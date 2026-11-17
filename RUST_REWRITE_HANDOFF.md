# Ar3mis Cloud - Rust Rewrite Handoff Document

## Executive Summary
**Project**: Complete Rust rewrite of Ar3mis Cloud platform  
**Previous Stack**: Python FastAPI (backend) + Next.js (frontend)  
**New Stack**: Axum/SQLx (backend) + Leptos WASM (frontend)  
**Deployment**: Backend -> Docker Compose on Rpi5 with PostgreSQL | Frontend -> Vercel (Leptos WASM/CSR)  
**Domain**: cloud.ar3mis.com (via Caddy reverse proxy)  
**GitHub Repo**: https://github.com/Ar3mis17/Rpi5-Cloud-Host  
**Last Commit**: 78eaa16 - "Rust rewrite: Axum + Leptos frontend/backend"

## Project Structure (Cargo Workspace)
/tmp/ar3mis-cloud-rust/
- Cargo.toml (Workspace root)
- backend/ (Axum backend)
  - Cargo.toml
  - src/main.rs, config.rs, db.rs, models.rs, auth.rs, errors.rs
  - middleware/auth.rs
  - api/mod.rs, files.rs, shares.rs, health.rs
  - services/storage.rs
- frontend/ (Leptos WASM frontend)
  - Cargo.toml, Trunk.toml
  - src/main.rs, lib.rs, app.rs, components.rs
  - components/layout.rs, login.rs, dashboard.rs, file_list.rs, not_found.rs
  - api.rs, store.rs, hooks.rs, utils.rs
- shared/ (Shared types)
  - Cargo.toml
  - src/lib.rs

## Backend Status: COMPLIES
All compiles: Axum 0.7, SQLx 0.8, JWT 9.3, Argon2, chunked uploads, shares, CORS, migrations

## Frontend Compilation Issues (CRITICAL)
Leptos 0.7 requires `use leptos::prelude::*;` in ALL files

### Files needing fixes:
1. frontend/src/store.rs - Add `use leptos::prelude::*;` at top (RwSignal, create_rw_signal, signal, expect_context, provide_context, spawn_local)
2. frontend/src/app.rs - Change imports to: `use leptos::prelude::*; use leptos_router::{Router, Routes, Route, Redirect};`
3. frontend/src/hooks.rs - Add `use leptos::prelude::*;`
4. frontend/src/components/login.rs - Add `use leptos::prelude::*; use leptos_router::hooks::use_navigate;`
5. frontend/src/components/layout.rs - Add `use leptos::children::Children;`
6. frontend/src/components/dashboard.rs - Add imports: leptos::prelude::*, leptos_router::hooks::use_navigate, js_sys::wasm_bindgen::JsCast, wasm_bindgen::closure::Closure, web_sys::HtmlInputElement
7. frontend/src/api.rs - COMPLETELY CORRUPT (only 13 bytes "test content") - needs full rewrite

## API Client Reference (frontend/src/api.rs)
Full implementation with: register, login, refresh_token, logout, get_me, list_files, create_folder, get_file, update_file, delete_file, download_file, get_storage_quota, initiate_upload, upload_chunk, get_upload_status, complete_upload, CreateFolderRequest, UpdateFileRequest

## Shared Types (shared/src/lib.rs)
User, FileEntry, RegisterRequest, LoginRequest, TokenResponse, RefreshRequest, AuthResponse, UserResponse, CreateFolderRequest, UpdateFileRequest, FileResponse, PaginatedResponse, UploadInitiateRequest, UploadInitiateResponse, UploadStatusResponse, CreateShareRequest, ShareLink, ShareResponse, StorageQuota

## Deployment Configuration
### Backend (Rpi5 - Docker Compose)
PostgreSQL + Backend services with environment variables for DATABASE_URL, JWT_SECRET, REFRESH_SECRET, etc.

### Frontend (Vercel)
Build command: `cd frontend && trunk build --release`, Output: frontend/dist

### Caddy Config (Rpi5)
Reverse proxy /api/* to localhost:8080

## Next Steps on Rpi5
1. git clone https://github.com/Ar3mis17/Rpi5-Cloud-Host.git
2. Fix frontend imports (sed commands provided in full document)
3. Rewrite api.rs completely
4. trunk build --release
5. cargo build --release (backend)
6. docker-compose up -d --build
7. Configure Caddy
8. Deploy frontend to Vercel

## Verification Checklist
- Backend health: curl https://cloud.ar3mis.com/health
- Frontend loads: Open https://cloud.ar3mis.com
- Register/Login/List files/Upload/Download/Create share

## Known Risks
- SQLx needs DATABASE_URL at compile time
- Leptos 0.7 breaking changes (prelude imports)
- WASM size optimization
- CORS matching exactly
- Chunked upload timeouts

## Support Commands
docker-compose logs -f backend, sqlx migrate run, trunk build --release, cargo test -p backend
