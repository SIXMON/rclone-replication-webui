# rclone-replication-ui

Interface web de gestion de réplication de fichiers basée sur [rclone](https://rclone.org/).

> **English version**: [README.md](README.md)

## Captures d'écran

| Liste des tâches | Détail et logs |
|:---:|:---:|
| ![Tâches](docs/screenshots/tasks-list.png) | ![Détail](docs/screenshots/task-detail.png) |

| Stockages distants | Formulaire de création |
|:---:|:---:|
| ![Stockages](docs/screenshots/remotes-list.png) | ![Formulaire](docs/screenshots/remote-form.png) |

## Fonctionnalités

- **Stockages distants** — Gestion des remotes rclone (S3, SFTP, FTP, SMB, local, et tous les types rclone)
- **Tâches de réplication** — Planification cron, déclenchement manuel, mode restauration (sync inverse)
- **Retry automatique** — Relance automatique en cas d'échec avec backoff linéaire configurable
- **Suivi en temps réel** — Progression SSE et logs en direct pendant l'exécution
- **Historique** — 100 dernières exécutions par tâche avec statistiques rclone (transferts, volume, erreurs)
- **Notifications** — Alertes via Apprise (Slack, Mattermost, email, webhooks) en cas d'erreur, succès ou tâche ignorée
- **Protection anti-chevauchement** — Une tâche déjà en cours ne peut pas être relancée

## Stack technique

| Composant | Technologie |
|-----------|-------------|
| Backend | Rust (Axum) |
| Frontend | React 18 + TypeScript + Vite + Tailwind CSS |
| Base de données | PostgreSQL (externe) |
| Planification | tokio-cron-scheduler |
| Réplication | rclone |
| Notifications | [apprise-go](https://github.com/unraid/apprise-go) |

## Architecture

```
┌─────────────────────────────────────┐
│  Container Frontend                 │
│  nginx → fichiers statiques         │
│  → proxy /api/* vers le backend     │
└─────────────────────────────────────┘
         ↕ HTTP/REST + SSE
┌─────────────────────────────────────┐
│  Container Backend                  │
│  Rust (Axum) + rclone + apprise-go  │
└─────────────────────────────────────┘
         ↕ SQL
┌─────────────────────────────────────┐
│  PostgreSQL (externe)               │
└─────────────────────────────────────┘
```

## Prérequis

### Avec Docker (recommandé)

- Docker + Docker Compose
- PostgreSQL accessible depuis les containers

### Sans Docker (développement)

- Rust 1.75+
- Node.js 18+
- PostgreSQL 14+
- rclone
- apprise-go (optionnel, pour les notifications)

## Lancement avec Docker

1. Créez un fichier `.env` à la racine :

```env
DATABASE_URL=postgresql://user:password@host.docker.internal:5432/rclone_ui
```

2. Lancez les containers :

```bash
docker compose up --build
```

3. Accédez à l'interface : [http://localhost](http://localhost)

## Lancement manuel (développement)

### Backend

```bash
cd backend
export DATABASE_URL="postgresql://user:password@localhost:5432/rclone_ui"
cargo run
```

Le serveur démarre sur `http://localhost:3000`. Les migrations sont appliquées automatiquement au démarrage.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Le serveur de développement Vite démarre sur `http://localhost:5173` et proxifie `/api/*` vers le backend.

## Variables d'environnement

| Variable | Description | Défaut |
|----------|-------------|--------|
| `DATABASE_URL` | URL de connexion PostgreSQL | **Requis** |
| `BIND_ADDR` | Adresse d'écoute du backend | `0.0.0.0:3000` |
| `RCLONE_BIN` | Chemin vers le binaire rclone | `rclone` |
| `APPRISE_BIN` | Chemin vers le binaire apprise | `apprise` |
| `RUST_LOG` | Niveau de logs | `info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn` |

## Structure du projet

```
rclone-replication-ui/
├── backend/
│   ├── src/
│   │   ├── entities/       # Entités SeaORM
│   │   ├── migration/      # Migrations BDD
│   │   ├── models/         # DTO requête/réponse
│   │   ├── routes/         # Handlers API
│   │   ├── services/       # Logique métier (rclone, scheduler, notifications)
│   │   └── sse/            # Server-Sent Events
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── pages/          # Pages React
│   │   ├── components/     # Composants UI
│   │   ├── hooks/          # React hooks
│   │   ├── api/            # Appels API
│   │   ├── types/          # Types TypeScript
│   │   └── config/         # Schémas de configuration
│   ├── nginx.conf
│   ├── package.json
│   └── Dockerfile
└── docker-compose.yml
```

## Commandes utiles

```bash
# Backend
cd backend && cargo build          # Compiler
cd backend && cargo clippy         # Lint
cd backend && cargo fmt            # Formater

# Frontend
cd frontend && npm run build       # Build production
cd frontend && npm run dev         # Serveur de développement
cd frontend && npm run lint        # Lint

# Docker
docker compose up --build          # Lancer
docker compose down                # Arrêter
docker compose logs -f backend     # Logs backend
```

## Licence

MIT
