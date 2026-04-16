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

- **Remote storage** — Manage rclone remotes (S3, SFTP, FTP, SMB, local, and all rclone types)
- **Replication tasks** — Cron scheduling, manual trigger, restore mode (reverse sync)
- **Auto-retry** — Automatic retry on failure with configurable linear backoff
- **Real-time monitoring** — SSE progress and live logs during execution
- **History** — Last 100 runs per task with rclone statistics (transfers, volume, errors)
- **Notifications** — Alerts via Apprise (Slack, Mattermost, email, webhooks) on error, success, or skipped tasks
- **Overlap protection** — A running task cannot be started again

## Tech stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (Axum) |
| Frontend | React 18 + TypeScript + Vite + Tailwind CSS |
| Database | PostgreSQL (external) |
| Scheduling | tokio-cron-scheduler |
| Replication | rclone |
| Notifications | [apprise-go](https://github.com/unraid/apprise-go) |

## Architecture

```
┌─────────────────────────────────────┐
│  Frontend container                 │
│  nginx → static files              │
│  → proxy /api/* to backend          │
└─────────────────────────────────────┘
         ↕ HTTP/REST + SSE
┌─────────────────────────────────────┐
│  Backend container                  │
│  Rust (Axum) + rclone + apprise-go  │
└─────────────────────────────────────┘
         ↕ SQL
┌─────────────────────────────────────┐
│  PostgreSQL (external)              │
└─────────────────────────────────────┘
```

## Prerequisites

### With Docker (recommended)

- Docker + Docker Compose
- PostgreSQL accessible from the containers

### Without Docker (development)

- Rust 1.75+
- Node.js 18+
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

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | **Required** |
| `BIND_ADDR` | Backend listen address | `0.0.0.0:3000` |
| `RCLONE_BIN` | Path to rclone binary | `rclone` |
| `APPRISE_BIN` | Path to apprise binary | `apprise` |
| `RUST_LOG` | Log levels | `info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn` |

## Project structure

```
rclone-replication-ui/
├── backend/
│   ├── src/
│   │   ├── entities/       # SeaORM entities
│   │   ├── migration/      # Database migrations
│   │   ├── models/         # Request/response DTOs
│   │   ├── routes/         # API handlers
│   │   ├── services/       # Business logic (rclone, scheduler, notifications)
│   │   └── sse/            # Server-Sent Events
│   ├── Cargo.toml
│   └── Dockerfile
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
└── docker-compose.yml
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
docker compose up --build          # Start
docker compose down                # Stop
docker compose logs -f backend     # Backend logs
```

## License

MIT
