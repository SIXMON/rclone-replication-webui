# rclone-replication-ui

Web interface for managing file replication powered by [rclone](https://rclone.org/).

> **Version française** : [README.fr.md](README.fr.md)

## Screenshots

| Tasks list | Task detail & logs |
|:---:|:---:|
| ![Tasks](docs/screenshots/tasks-list.png) | ![Detail](docs/screenshots/task-detail.png) |

| Remote storages | New remote form |
|:---:|:---:|
| ![Remotes](docs/screenshots/remotes-list.png) | ![Form](docs/screenshots/remote-form.png) |

## Features

- **Remote storage** — Manage rclone remotes with guided forms for the most common types (S3, SFTP, FTP, SMB, Azure Blob Storage, SharePoint, local), and key/value editor for advanced types
- **Replication tasks** — Cron scheduling (5 or 6 fields, macros like `@daily`), live cron parser with next-run preview, manual trigger, restore mode (reverse sync)
- **Auto-retry** — Automatic retry on failure with configurable linear backoff
- **Real-time monitoring** — SSE progress and live logs during execution, global SSE for cross-tab state updates
- **History** — Last 100 runs per task with rclone statistics (transfers, volume, checks, deletes, errors), expandable inline logs
- **Notifications** — Markdown-formatted alerts via [apprise-go](https://github.com/unraid/apprise-go) (Slack, Mattermost, email, webhooks) on error, success, or skipped tasks
- **Overlap protection** — A running task cannot be started again; cron triggers during a running task are logged as "skipped"
- **Secret Manager** — Optional [Scaleway Secret Manager](https://www.scaleway.com/en/secret-manager/) integration to store sensitive credentials outside the database

## Tech stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (Axum 0.8) |
| ORM | SeaORM 1.x |
| Frontend | React 19 + TypeScript + Vite 8 + Tailwind CSS 4 |
| Database | PostgreSQL (external or bundled via `docker-compose.postgres.yml`) |
| Scheduling | tokio-cron-scheduler |
| Replication | rclone |
| Notifications | [apprise-go](https://github.com/unraid/apprise-go) |
| Secret storage (optional) | Scaleway Secret Manager |

## Architecture

```
┌─────────────────────────────────────┐
│  Frontend container                 │
│  nginx → static files               │
│  → proxy /api/* to backend          │
└─────────────────────────────────────┘
         ↕ HTTP/REST + SSE
┌─────────────────────────────────────┐
│  Backend container (FROM scratch)   │
│  Rust (Axum) + rclone + apprise-go  │
└─────────────────────────────────────┘
         ↕ SQL              ↕ HTTPS (optional)
┌──────────────────┐    ┌────────────────────────┐
│  PostgreSQL      │    │  Scaleway Secret Mgr   │
└──────────────────┘    └────────────────────────┘
```

## Prerequisites

### With Docker (recommended)

- Docker + Docker Compose
- PostgreSQL (or use the bundled compose file)

### Without Docker (development)

- Rust 1.75+
- Node.js 24+ (see [`frontend/.node-version`](frontend/.node-version))
- PostgreSQL 14+
- rclone
- apprise-go (optional, for notifications)

## Running with Docker

### Quick start (with PostgreSQL included)

No external database needed — everything runs in Docker:

```bash
docker compose -f docker-compose.postgres.yml up --build
```

Open the interface: [http://localhost](http://localhost)

### With an existing PostgreSQL

1. Create a `.env` file at the project root:

```env
DATABASE_URL=postgresql://user:password@host.docker.internal:5432/rclone_ui
```

2. Start the containers:

```bash
docker compose up --build
```

3. Open the interface: [http://localhost](http://localhost)

## Running manually (development)

### Backend

```bash
cd backend
export DATABASE_URL="postgresql://user:password@localhost:5432/rclone_ui"
cargo run
```

The server starts on `http://localhost:3000`. Migrations run automatically on startup.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The Vite dev server starts on `http://localhost:5173` and proxies `/api/*` to the backend.

## Environment variables

### Core

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | **Required** |
| `BIND_ADDR` | Backend listen address | `0.0.0.0:3000` |
| `RCLONE_BIN` | Path to rclone binary | `rclone` |
| `APPRISE_BIN` | Path to apprise binary | `apprise` |
| `RUST_LOG` | Log levels | `info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn` |

### Scaleway Secret Manager (optional)

When enabled, sensitive credentials (passwords, API secrets, keys) are stored in [Scaleway Secret Manager](https://www.scaleway.com/en/secret-manager/) instead of the database. Existing credentials are migrated automatically on startup.

| Variable | Description | Default |
|----------|-------------|---------|
| `SCW_SECRET_MANAGER_ENABLED` | Set to `true` to enable | `false` |
| `SCW_ACCESS_KEY` | IAM access key | — |
| `SCW_SECRET_KEY` | IAM secret key | — |
| `SCW_PROJECT_ID` | Scaleway project UUID | — |
| `SCW_DEFAULT_REGION` | Region | `fr-par` |
| `SCW_SECRET_PATH` | Path prefix for secrets | `/rclone-ui` |

**Setup steps on the Scaleway console:**

1. Activate **Secret Manager** in your project (region `fr-par` recommended)
2. Create an **IAM API key** with the permission set `SecretManagerFullAccess`
3. Copy the access key, secret key, and project ID into your `.env` or `docker-compose.yml`

## Project structure

```
rclone-replication-ui/
├── backend/
│   ├── src/
│   │   ├── entities/       # SeaORM entities
│   │   ├── migration/      # Database migrations
│   │   ├── models/         # Request/response DTOs
│   │   ├── routes/         # API handlers
│   │   ├── services/       # Business logic
│   │   │   └── secrets/    # SecretStore abstraction (Scaleway / no-op)
│   │   └── sse/            # Server-Sent Events
│   ├── Cargo.toml
│   └── Dockerfile          # FROM scratch image
├── frontend/
│   ├── src/
│   │   ├── pages/          # React pages
│   │   ├── components/     # UI components
│   │   ├── hooks/          # React hooks
│   │   ├── api/            # API calls
│   │   ├── types/          # TypeScript types
│   │   └── config/         # Configuration schemas
│   ├── nginx.conf
│   ├── package.json
│   └── Dockerfile
├── docker-compose.yml             # External PostgreSQL
└── docker-compose.postgres.yml    # Bundled PostgreSQL 18
```

## Useful commands

```bash
# Backend
cd backend && cargo build          # Build
cd backend && cargo clippy         # Lint
cd backend && cargo fmt            # Format

# Frontend
cd frontend && npm run build       # Production build
cd frontend && npm run dev         # Dev server
cd frontend && npm run lint        # Lint

# Docker
docker compose up --build                              # External PostgreSQL
docker compose -f docker-compose.postgres.yml up --build  # Bundled PostgreSQL
docker compose down                                    # Stop
docker compose logs -f backend                         # Backend logs
```

## License

MIT
