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

- **Stockages distants** — Gestion des remotes rclone avec formulaires guidés pour les types courants (S3, SFTP, FTP, SMB, Azure Blob Storage, SharePoint, local) et éditeur clé/valeur pour les types avancés
- **Tâches de réplication** — Planification cron (5 ou 6 champs, macros comme `@daily`), parseur cron en direct avec aperçu des prochaines exécutions, déclenchement manuel, mode restauration (sync inverse)
- **Retry automatique** — Relance automatique en cas d'échec avec backoff linéaire configurable
- **Suivi en temps réel** — Progression SSE et logs en direct pendant l'exécution, SSE global pour la mise à jour cross-tab des états
- **Historique** — 100 dernières exécutions par tâche avec statistiques rclone (transferts, volume, vérifications, suppressions, erreurs), logs dépliables inline
- **Notifications** — Alertes formatées en markdown via [apprise-go](https://github.com/unraid/apprise-go) (Slack, Mattermost, email, webhooks) en cas d'erreur, succès ou tâche ignorée
- **Protection anti-chevauchement** — Une tâche déjà en cours ne peut pas être relancée ; les déclenchements cron pendant une exécution sont enregistrés comme « ignorés »
- **Secret Manager** — Intégration optionnelle avec [Scaleway Secret Manager](https://www.scaleway.com/en/secret-manager/) pour stocker les credentials sensibles hors de la base de données

## Stack technique

| Composant | Technologie |
|-----------|-------------|
| Backend | Rust (Axum 0.8) |
| ORM | SeaORM 1.x |
| Frontend | React 19 + TypeScript + Vite 8 + Tailwind CSS 4 |
| Base de données | PostgreSQL (externe ou inclus via `docker-compose.postgres.yml`) |
| Planification | tokio-cron-scheduler |
| Réplication | rclone |
| Notifications | [apprise-go](https://github.com/unraid/apprise-go) |
| Stockage des secrets (optionnel) | Scaleway Secret Manager |

## Architecture

```
┌─────────────────────────────────────┐
│  Container Frontend                 │
│  nginx → fichiers statiques         │
│  → proxy /api/* vers le backend     │
└─────────────────────────────────────┘
         ↕ HTTP/REST + SSE
┌─────────────────────────────────────┐
│  Container Backend (FROM scratch)   │
│  Rust (Axum) + rclone + apprise-go  │
└─────────────────────────────────────┘
         ↕ SQL              ↕ HTTPS (optionnel)
┌──────────────────┐    ┌────────────────────────┐
│  PostgreSQL      │    │  Scaleway Secret Mgr   │
└──────────────────┘    └────────────────────────┘
```

## Prérequis

### Avec Docker (recommandé)

- Docker + Docker Compose
- PostgreSQL (ou utiliser le compose avec PostgreSQL inclus)

### Sans Docker (développement)

- Rust 1.75+
- Node.js 24+ (voir [`frontend/.node-version`](frontend/.node-version))
- PostgreSQL 14+
- rclone
- apprise-go (optionnel, pour les notifications)

## Lancement avec Docker

### Démarrage rapide (PostgreSQL inclus)

Aucune base de données externe nécessaire — tout tourne dans Docker :

```bash
docker compose -f docker-compose.postgres.yml up --build
```

Accédez à l'interface : [http://localhost](http://localhost)

### Avec un PostgreSQL existant

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

### Principales

| Variable | Description | Défaut |
|----------|-------------|--------|
| `DATABASE_URL` | URL de connexion PostgreSQL | **Requis** |
| `BIND_ADDR` | Adresse d'écoute du backend | `0.0.0.0:3000` |
| `RCLONE_BIN` | Chemin vers le binaire rclone | `rclone` |
| `APPRISE_BIN` | Chemin vers le binaire apprise | `apprise` |
| `RUST_LOG` | Niveau de logs | `info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn` |

### Scaleway Secret Manager (optionnel)

Quand activé, les credentials sensibles (mots de passe, secrets API, clés) sont stockés dans [Scaleway Secret Manager](https://www.scaleway.com/en/secret-manager/) au lieu de la base de données. Les credentials existants sont migrés automatiquement au démarrage.

| Variable | Description | Défaut |
|----------|-------------|--------|
| `SCW_SECRET_MANAGER_ENABLED` | Mettre à `true` pour activer | `false` |
| `SCW_ACCESS_KEY` | Access key IAM | — |
| `SCW_SECRET_KEY` | Secret key IAM | — |
| `SCW_PROJECT_ID` | UUID du projet Scaleway | — |
| `SCW_DEFAULT_REGION` | Région | `fr-par` |
| `SCW_SECRET_PATH` | Préfixe de path pour organiser les secrets | `/rclone-ui` |

**Configuration côté console Scaleway :**

1. Activer **Secret Manager** dans votre projet (région `fr-par` recommandée)
2. Créer une **clé API IAM** avec le permission set `SecretManagerFullAccess`
3. Copier l'access key, la secret key et l'ID du projet dans votre `.env` ou `docker-compose.yml`

## Structure du projet

```
rclone-replication-ui/
├── backend/
│   ├── src/
│   │   ├── entities/       # Entités SeaORM
│   │   ├── migration/      # Migrations BDD
│   │   ├── models/         # DTO requête/réponse
│   │   ├── routes/         # Handlers API
│   │   ├── services/       # Logique métier
│   │   │   └── secrets/    # Abstraction SecretStore (Scaleway / no-op)
│   │   └── sse/            # Server-Sent Events
│   ├── Cargo.toml
│   └── Dockerfile          # Image FROM scratch
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
├── docker-compose.yml             # PostgreSQL externe
└── docker-compose.postgres.yml    # PostgreSQL 18 inclus
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
docker compose up --build                              # PostgreSQL externe
docker compose -f docker-compose.postgres.yml up --build  # PostgreSQL inclus
docker compose down                                    # Arrêter
docker compose logs -f backend                         # Logs backend
```

## Licence

MIT
