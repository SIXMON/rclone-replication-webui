# rclone-replication-ui

Interface web de gestion de réplication de fichiers basée sur [rclone](https://rclone.org/).

## Vue d'ensemble

Application fullstack permettant de piloter rclone via une UI web : gestion des stockages distants, planification de tâches de réplication, notifications, logs et monitoring.

## Architecture

```
┌─────────────────────────────────────────────┐
│  Container Frontend                         │
│  nginx → fichiers statiques compilés        │
│  → proxy /api/* vers le backend             │
└─────────────────────────────────────────────┘
         ↕ HTTP/REST
┌─────────────────────────────────────────────┐
│  Container Backend                          │
│  Rust (Axum ou Actix-web)                   │
│  + rclone (binaire embarqué)                │
│  + cron scheduler (crontab ou tokio-cron)   │
└─────────────────────────────────────────────┘
         ↕ SQL
┌─────────────────────────────────────────────┐
│  PostgreSQL (externe, hors Docker Compose)  │
└─────────────────────────────────────────────┘
```

## Stack technique

- **Backend** : Rust
- **Frontend** : à compiler en fichiers statiques, servis par nginx (ex. React/TypeScript, Vue/TypeScript ou Svelte)
- **Base de données** : PostgreSQL (hors containers)
- **Scheduling** : cron (crontab système ou scheduler interne Rust)
- **Notifications** : [Apprise](https://github.com/caronc/apprise) (client) — compatible emails, webhooks (Mattermost, Slack, etc.)
- **Réplication** : rclone (binaire embarqué dans le container backend)

## Conteneurs Docker

Deux containers distincts (définis dans `docker-compose.yml`) :

| Container    | Contenu                                                      |
|--------------|--------------------------------------------------------------|
| `frontend`   | nginx + build statique du frontend                           |
| `backend`    | Rust binary + rclone + Apprise client + scheduler            |

La base de données PostgreSQL est **externe** (non gérée par Docker Compose).

## Fonctionnalités

### Stockages distants
- CRUD des remotes rclone (Azure Files, S3-compatible, SMB, FTP, Google Drive, etc.)
- Support de tous les types de remotes rclone via configuration dynamique

### Tâches de réplication
- Création de tâches source → destination (avec récurrence cron)
- Modification des paramètres d'une tâche (hors source/destination)
- Activation / désactivation d'une tâche
- Déclenchement manuel d'une tâche
- Mode "restauration" : synchronisation en inversant source et destination
- Suivi en temps réel de la progression si une tâche est en cours

### Logs & historique
- Historique des 100 dernières exécutions par tâche (état : succès / échec, durée, logs)

### Notifications
- Configuration de canaux Apprise (email, webhook Mattermost, Slack, etc.)
- Envoi de notifications en cas d'erreur lors d'une tâche

## Structure du projet (cible)

```
rclone-replication-ui/
├── backend/               # Crate Rust
│   ├── src/
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/              # App frontend (React/Vue/Svelte)
│   ├── src/
│   ├── package.json
│   ├── nginx.conf
│   └── Dockerfile
├── docker-compose.yml
└── CLAUDE.md
```

## Conventions de développement

### Backend Rust
- ORM : `sqlx` avec migrations PostgreSQL dans `backend/migrations/`
- Format : `cargo fmt` avant chaque commit
- Lint : `cargo clippy -- -D warnings`

### Frontend
- Format + lint avant chaque commit
- Les appels API passent tous par `/api/` (proxifié par nginx vers le backend)

### Base de données
- Toutes les migrations sont versionnées dans `backend/migrations/`
- Ne jamais modifier une migration existante, toujours en créer une nouvelle

## Commandes utiles

```bash
# Backend
cd backend && cargo build
cd backend && cargo test
cd backend && cargo clippy

# Frontend
cd frontend && npm install
cd frontend && npm run build
cd frontend && npm run dev

# Docker
docker compose up --build
docker compose down
```
