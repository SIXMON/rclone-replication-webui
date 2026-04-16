# ─── Build stage ─────────────────────────────────────────────────────────────
FROM node:alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts

COPY . .
RUN npm run build

# ─── Runtime stage (nginx non-root) ──────────────────────────────────────────
FROM nginx:alpine

# Supprimer la config par défaut et le contenu statique d'exemple
RUN rm -rf /usr/share/nginx/html/* /etc/nginx/conf.d/default.conf

# Copier les fichiers buildés et la config nginx
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Créer les sous-dossiers de cache nginx et fixer les permissions pour non-root
RUN mkdir -p /var/cache/nginx/client_temp \
             /var/cache/nginx/proxy_temp \
             /var/cache/nginx/fastcgi_temp \
             /var/cache/nginx/uwsgi_temp \
             /var/cache/nginx/scgi_temp \
    && chown -R nginx:nginx /usr/share/nginx/html \
                            /var/cache/nginx \
                            /var/log/nginx \
    && touch /var/run/nginx.pid && chown nginx:nginx /var/run/nginx.pid

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD wget -qO /dev/null http://localhost:80/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
