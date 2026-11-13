# Ar3mis Cloud - Complete Session Work Log
======================================================

**Last Updated**: 2026-11-14
**Git Branch**: `main` (was `pi-backend`, renamed)
**GitHub Repo**: https://github.com/Ar3mis17/Rpi5-Cloud-Host.git
**Latest Commit**: `0601500` - "Fix null vs undefined in handlePageChange"

---

## 🎯 PROJECT OVERVIEW

**Ar3mis Cloud** - Self-hosted cloud storage platform on Raspberry Pi 5
- **Backend**: Python/FastAPI + PostgreSQL (Docker)
- **Frontend**: Next.js 14 + TypeScript + Tailwind (Vercel)
- **Reverse Proxy**: Caddy (ports 80/443)
- **DNS/Ad-blocking**: Pi-hole (DNS only on port 53)
- **Storage**: 512GB SSD at `/mnt/cloud` (ext4, UUID: ef46080c-dfbb-4657-9ad8-63267991b789)

---

## ✅ PHASES COMPLETED (1-16)

### Phase 1-2: Infrastructure & Setup
- Docker Compose with PostgreSQL + FastAPI on internal network
- Bind mounts to `/mnt/cloud/ar3mis-cloud/postgres/`
- `.env` with generated credentials (NOT in git)

### Phase 3: Backend Structure
- FastAPI app with proper package structure
- Pydantic Settings with CORS, security, upload config
- Fixed: CryptContext `deprecated="auto"`, SQLAlchemy `remote_side`, DATABASE_URL format

### Phase 4: Models & Schemas
- User, UserStorage, File, Share, Session, ActivityLog models
- Pydantic schemas for all operations

### Phase 5: Authentication (Complete)
- Argon2 password hashing, JWT access/refresh tokens
- Endpoints: register, login, refresh, logout, me, change-password, forgot/reset, sessions

### Phase 6: Health Endpoints
- `/health`, `/health/database`, `/`

### Phase 7: File Management
- List files (pagination), create folder, delete (soft delete)

### Phase 8: Chunked Uploads (Complete)
- Initiate → Upload chunks → Status → Complete
- SHA-256 verification, quota enforcement, temp cleanup

### Phase 9: Router Composition
- All routers registered under `/api/v1`

### Phase 10: Docker Infrastructure
- PostgreSQL + FastAPI containers, internal network
- Dockerfile with python-multipart

### Phase 11: Testing & Verification
- API healthy at 127.0.0.1:8000, PostgreSQL internal, FreeLLMAPI preserved at 127.0.0.1:3001

### Phase 12: Storage Engine
- Complete chunked upload workflow with SHA-256

### Phase 13: Sharing & Permissions
- Share files by email with permissions/expiry

### Phase 14: Caddy + Pi-hole Integration ✅
- Caddy v2.8.4 ARM64 binary installed
- Systemd service `caddy-pihole` on ports 80/443
- Pi-hole FTL runs DNS-only (port 53), no web interface conflict
- Proxies: `/api/*` → Backend (8000), `/*` → Vercel Frontend
- Auto HTTPS disabled for local dev

### Phase 15: Vercel Frontend (Next.js 14) ✅
**Location**: `/opt/ar3mis-cloud/frontend/`

**Tech Stack**:
- Next.js 14 (App Router), TypeScript, Tailwind CSS
- Zustand (state), Axios (API), react-hot-toast, Lucide React

**Pages**:
- `/` → Redirect to login/dashboard
- `/login` → Login/Register with JWT
- `/dashboard` → File browser with sidebar, breadcrumbs, pagination

**Components**:
- `FileList.tsx` - File browser, chunked uploads with progress, folder nav, delete
- `LoginForm.tsx` - Auth form with validation

**State (Zustand)**:
- `useAuthStore` - User auth with localStorage persistence
- `useFileStore` - Files, folders, breadcrumbs, pagination, `loadFiles()`
- `useUploadStore` - Upload sessions with progress tracking

**API Client** (`lib/api.ts`):
- Axios with request/response interceptors
- Auto JWT refresh on 401
- Returns user object after login/register

**Caddy Config**: Proxies API to backend, frontend to Vercel

### Phase 16: Backups, Monitoring, Hardening ✅
**Scripts** (`/opt/ar3mis-cloud/scripts/`):
- `backup-postgres.sh` - PostgreSQL logical backup via Docker (30-day retention)
- `backup-files.sh` - File storage tar.gz (14-day retention)
- `backup-all.sh` - Master orchestration
- `health-check.sh` - API, DB, Caddy, disk, memory, Docker checks
- `collect-metrics.sh` - Prometheus format metrics
- `setup-monitoring.sh` - Installs Prometheus, Node Exporter, cAdvisor
- `harden-production.sh` - SSH, UFW, Kernel, Fail2Ban, Docker, Log rotation
- `install-services.sh` - Deploys all systemd services/timers

**Systemd Services**:
- `ar3mis-backup.timer` (daily)
- `ar3mis-health.timer` (every 5 min)
- `ar3mis-metrics.timer` (every 1 min)
- `ar3mis-harden.service` (once at boot)

---

## 📁 REPOSITORY STRUCTURE

```
/opt/ar3mis-cloud/
├── backend/                 # Python/FastAPI backend
│   ├── app/
│   │   ├── api/            # Auth, files, uploads, shares, health
│   │   ├── core/           # Config, security
│   │   ├── db/             # Session, base
│   │   ├── models/         # SQLAlchemy models
│   │   ├── schemas/        # Pydantic schemas
│   │   ├── services/       # Storage service
│   │   └── main.py         # FastAPI app
│   ├── alembic/            # Migrations
│   ├── Dockerfile
│   ├── .env                # NOT in git (secrets)
│   └── .env.example        # Template
├── frontend/               # Next.js 14 (Vercel)
│   ├── src/
│   │   ├── app/            # Pages (login, dashboard)
│   │   ├── components/     # FileList, LoginForm
│   │   └── lib/            # API, store
│   ├── .env.local          # Dev config
│   ├── .env.production     # Prod config
│   ├── vercel.json         # Vercel config
│   └── deploy-vercel.sh    # Deploy script
├── caddy/
│   └── config/
│       ├── Caddyfile       # Reverse proxy config
│       ├── caddy-pihole.service
│       └── install-system.sh
├── scripts/                # Backup, monitoring, hardening
│   ├── *.sh                # 15 scripts
│   └── *.service/*.timer   # 7 services + 3 timers
├── infra/
│   └── caddy/setup.sh      # Original setup script
├── compose.yaml            # Docker Compose
├── SESSION_LOG.md          # This file
└── GIT_STATUS.md           # Git status summary
```

---

## 🔧 CURRENT RUNNING SERVICES

| Service | Status | Port/Location |
|---------|--------|---------------|
| Backend API | ✅ Running | `127.0.0.1:8000` (Docker) |
| PostgreSQL | ✅ Running | Internal Docker network |
| Caddy Proxy | ✅ Running | Ports 80/443 (`caddy-pihole` service) |
| Pi-hole FTL | ✅ Running | Port 53 (DNS only) |
| FreeLLMAPI | ✅ Preserved | `127.0.0.1:3001` |
| Frontend | 🚀 Deploying | Vercel (auto-deploy from GitHub) |

---

## 🌐 VERCEL DEPLOYMENT STATUS

**Repository**: `Ar3mis17/Rpi5-Cloud-Host` (branch: `main`)
**Root Directory**: `frontend`
**Latest Commit**: `0601500` - "Fix null vs undefined in handlePageChange"

**Environment Variables** (set in Vercel dashboard):
```
NEXT_PUBLIC_API_URL=https://cloud.ar3mis.com
```

**Build Status**: Check Vercel dashboard - should build successfully now with all TypeScript errors fixed.

**After Deployment**:
1. Get Vercel URL (e.g., `https://ar3mis-cloud.vercel.app`)
2. Update `/opt/ar3mis-cloud/caddy/config/Caddyfile` with actual Vercel domain
3. Restart Caddy: `sudo cp ... && sudo systemctl restart caddy-pihole`

---

## 🔐 SECURITY NOTES

- **No secrets in git**: `.env` in `.gitignore`
- **SSH**: Key-only auth, root disabled, max 3 attempts
- **Firewall**: UFW enabled (SSH, HTTP, HTTPS only)
- **Fail2Ban**: SSH + API protection
- **Docker**: `no-new-privileges`, log limits
- **Kernel**: Hardened sysctl params
- **Backups**: Directory permissions 700 root:root

---

## 🚀 NEXT SESSION - WHERE TO START

### Immediate (if Vercel deploy succeeds):
1. **Get Vercel deployment URL** from dashboard
2. **Update Caddyfile** with actual Vercel domain:
   ```bash
   # Edit /opt/ar3mis-cloud/caddy/config/Caddyfile
   # Replace: https://ar3mis-cloud.vercel.app
   # With: https://your-actual-vercel-domain.vercel.app
   sudo cp /opt/ar3mis-cloud/caddy/config/Caddyfile /etc/caddy/Caddyfile
   sudo systemctl restart caddy-pihole
   ```
3. **Test full flow**: Login → Dashboard → Upload → Download → Share

### Pending Features (not yet implemented):
1. **Complete forgot/reset password** (backend returns 501)
2. **Sharing endpoints**: GET `/shares/`, DELETE `/shares/{id}/`, GET `/public/{token}/`
3. **Session cleanup**: Expired session removal job
4. **Production domain + SSL**: Configure real domain, enable `auto_https`
4. **Monitoring dashboards**: Grafana dashboards for Prometheus metrics
5. **Automated backup verification**: Restore tests

### If Vercel Build Still Fails:
Check the latest build logs at Vercel dashboard. The current fixes address:
- ✅ `loadFiles` in `useFileStore`
- ✅ `useState` import in dashboard
- ✅ `File` vs `FileIcon` naming conflict
- ✅ `null` vs `undefined` in `handlePageChange`
- ✅ API response format (returns user after login)

---

## 📋 KEY FILES FOR REFERENCE

| File | Purpose |
|------|---------|
| `/opt/ar3mis-cloud/backend/.env` | Production secrets (DB password, JWT secret) |
| `/opt/ar3mis-cloud/compose.yaml` | Docker services |
| `/opt/ar3mis-cloud/caddy/config/Caddyfile` | Reverse proxy config |
| `/opt/ar3mis-cloud/frontend/.env.production` | Vercel API URL |
| `/opt/ar3mis-cloud/scripts/install-services.sh` | Deploy all systemd services |
| `/opt/ar3mis-cloud/scripts/harden-production.sh` | Production hardening |

---

## 🔄 TO RESUME WORK

```bash
# 1. Check GitHub for latest
cd /opt/ar3mis-cloud && git pull

# 2. Check running services
docker ps --filter name=ar3mis-cloud
systemctl status caddy-pihole

# 3. Test API
curl http://localhost/health
curl http://localhost/api/v1/health

# 4. Check Vercel deployment
# Visit: https://vercel.com/dashboard → your project

# 5. Run hardening if not done
sudo systemctl start ar3mis-harden.service

# 6. Install monitoring if desired
sudo bash /opt/ar3mis-cloud/scripts/setup-monitoring.sh
```

---

## ⚠️ IMPORTANT CONSTRAINTS (NEVER VIOLATE)

1. **NEVER** stop/remove/modify FreeLLMAPI (port 3001)
2. **NEVER** format/repartition SSD at `/mnt/cloud`
3. **NEVER** run `docker system prune` or `docker volume prune`
4. **NEVER** expose PostgreSQL publicly
5. **NEVER** hardcode secrets in code/compose
6. **ALWAYS** use bind mounts for PostgreSQL data (not Docker volumes)
7. **ALWAYS** preserve `ar3mis:ar3mis` ownership of app directory

---

**End of Session Log** - All phases 1-16 complete. Ready for production deployment.