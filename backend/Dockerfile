# ─── Build stage (Alpine + musl = binaire statique) ──────────────────────────
FROM rust:alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

# Cache des dépendances Cargo (layer séparé)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Compilation de l'application
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ─── Rclone stage ────────────────────────────────────────────────────────────
FROM alpine:3 AS rclone-dl

RUN apk add --no-cache curl unzip

ARG TARGETARCH=amd64
RUN curl -fsSL "https://downloads.rclone.org/rclone-current-linux-${TARGETARCH}.zip" -o /tmp/rclone.zip \
    && unzip -q /tmp/rclone.zip -d /tmp \
    && install -m 755 /tmp/rclone-*/rclone /usr/local/bin/rclone

# ─── Apprise-go stage (binaire Go statique) ──────────────────────────────────
FROM alpine:3 AS apprise-dl

RUN apk add --no-cache curl

ARG TARGETARCH=amd64
ARG APPRISE_VERSION=v0.1.11
RUN curl -fsSL "https://github.com/unraid/apprise-go/releases/download/${APPRISE_VERSION}/apprise-go-linux-${TARGETARCH}" \
    -o /usr/local/bin/apprise \
    && chmod +x /usr/local/bin/apprise

# ─── Prep stage (créer les fichiers nécessaires pour scratch) ────────────────
FROM alpine:3 AS prep

# Créer un utilisateur non-root pour l'image scratch
RUN addgroup -S appuser && adduser -S -G appuser -H -s /sbin/nologin appuser \
    && mkdir -p /app && chown appuser:appuser /app

# ─── Image finale (FROM scratch — taille minimale absolue) ───────────────────
FROM scratch

# Certificats TLS
COPY --from=prep /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Utilisateur non-root
COPY --from=prep /etc/passwd /etc/passwd
COPY --from=prep /etc/group /etc/group
COPY --from=prep --chown=appuser:appuser /app /app

# Les 3 binaires
COPY --from=builder /app/target/release/rclone-replication-ui /usr/local/bin/rclone-replication-ui
COPY --from=rclone-dl /usr/local/bin/rclone /usr/local/bin/rclone
COPY --from=apprise-dl /usr/local/bin/apprise /usr/local/bin/apprise

WORKDIR /app
USER appuser

EXPOSE 3000

ENV BIND_ADDR=0.0.0.0:3000 \
    RCLONE_BIN=/usr/local/bin/rclone \
    APPRISE_BIN=/usr/local/bin/apprise \
    RUST_LOG=info \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

CMD ["/usr/local/bin/rclone-replication-ui"]
