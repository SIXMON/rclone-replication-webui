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
- **Secret Manager** — Optional integration with multiple providers to store sensitive credentials outside the database: Scaleway, AWS, Azure Key Vault, Google Cloud, HashiCorp Vault, Infisical, Doppler

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
| Secret storage (optional) | Scaleway / AWS / Azure / GCP / Vault / Infisical / Doppler |

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
┌──────────────────┐    ┌────────────────────────────┐
│  PostgreSQL      │    │  Secret Manager (pluggable)│
│                  │    │  Scaleway / AWS / Azure /  │
│                  │    │  GCP / Vault / Infisical / │
│                  │    │  Doppler                   │
└──────────────────┘    └────────────────────────────┘
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

### Secret Manager (optional)

When enabled, sensitive credentials (passwords, API secrets, keys) are stored in an external Secret Manager instead of the database. Existing credentials are migrated automatically on startup.

The provider is selected via `SECRET_MANAGER_PROVIDER`. Supported values:

| Provider | Value | Auth |
|----------|-------|------|
| Scaleway Secret Manager | `scaleway` | API key |
| AWS Secrets Manager | `aws` | Standard AWS credentials chain |
| Azure Key Vault | `azure` | Service Principal |
| Google Cloud Secret Manager | `gcp` | Application Default Credentials |
| HashiCorp Vault | `vault` | Token (KV v2) |
| Infisical | `infisical` | Universal Auth |
| Doppler | `doppler` | Service Token |

Set `SECRET_MANAGER_PROVIDER=none` (default) to keep credentials in the database.

#### Scaleway

| Variable | Default |
|----------|---------|
| `SCW_SECRET_KEY` | — |
| `SCW_PROJECT_ID` | — |
| `SCW_DEFAULT_REGION` | `fr-par` |
| `SCW_SECRET_PATH` | `/rclone-ui` |

#### AWS

Uses the standard [AWS credentials provider chain](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credentials.html) (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`, IAM roles on EC2/ECS/EKS, IRSA, etc.).

| Variable | Default |
|----------|---------|
| `AWS_REGION` | `eu-west-1` |
| `AWS_SECRET_PREFIX` | `rclone-ui/` |

#### Azure Key Vault

Requires a Service Principal with `Get`, `Set`, `Delete` permissions on the vault.

| Variable | Default |
|----------|---------|
| `AZURE_TENANT_ID` | — |
| `AZURE_CLIENT_ID` | — |
| `AZURE_CLIENT_SECRET` | — |
| `AZURE_VAULT_URL` (e.g. `https://myvault.vault.azure.net`) | — |

#### Google Cloud Secret Manager

Uses [Application Default Credentials](https://cloud.google.com/docs/authentication/application-default-credentials). Set `GOOGLE_APPLICATION_CREDENTIALS` to a service account JSON file, or rely on the GCE/GKE metadata server.

| Variable | Default |
|----------|---------|
| `GCP_PROJECT_ID` | — |

#### HashiCorp Vault

Uses the KV v2 secrets engine.

| Variable | Default |
|----------|---------|
| `VAULT_ADDR` (e.g. `http://vault.local:8200`) | — |
| `VAULT_TOKEN` | — |
| `VAULT_MOUNT_PATH` | `secret` |
| `VAULT_PATH_PREFIX` | `rclone-ui` |

#### Infisical

Uses [Universal Auth](https://infisical.com/docs/documentation/platform/identities/universal-auth) (machine identity).

| Variable | Default |
|----------|---------|
| `INFISICAL_HOST` | `https://app.infisical.com` |
| `INFISICAL_CLIENT_ID` | — |
| `INFISICAL_CLIENT_SECRET` | — |
| `INFISICAL_PROJECT_ID` | — |
| `INFISICAL_ENVIRONMENT` | `prod` |
| `INFISICAL_SECRET_PATH` | `/rclone-ui` |

#### Doppler

| Variable | Default |
|----------|---------|
| `DOPPLER_TOKEN` (Service Token) | — |
| `DOPPLER_PROJECT` | — |
| `DOPPLER_CONFIG` | `prd` |

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
