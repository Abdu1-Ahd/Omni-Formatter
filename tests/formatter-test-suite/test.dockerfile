# syntax=docker/dockerfile:1

# ── CASE 1: Base image and metadata ───────────────────────────────────────
FROM   node:20-alpine   AS builder

LABEL maintainer="alice@example.com"
LABEL version=  "1.0.0"
LABEL description=   "Test Dockerfile for OmniFormatter"

# ── CASE 2: ARG and ENV — mixed spacing ───────────────────────────────────
ARG   NODE_ENV=production
ARG   PORT=3000
ARG BUILD_DATE

ENV NODE_ENV=$NODE_ENV   \
    PORT=$PORT   \
    APP_DIR=/app

# ── CASE 3: WORKDIR and COPY ─────────────────────────────────────────────
WORKDIR   /app

COPY   package*.json   ./
COPY   tsconfig.json   ./

# ── CASE 4: RUN with multi-line — indentation ─────────────────────────────
RUN npm ci --only=production && \
    npm cache clean --force && \
    rm -rf /tmp/*

RUN   apk add --no-cache   \
    git   \
    curl   \
    bash

# ── CASE 5: COPY source and build ─────────────────────────────────────────
COPY   src/   ./src/
COPY   public/   ./public/

RUN npm run build

# ── CASE 6: Multi-stage build ─────────────────────────────────────────────
FROM   node:20-alpine   AS runtime

WORKDIR   /app

COPY   --from=builder   /app/dist   ./dist
COPY   --from=builder   /app/node_modules   ./node_modules
COPY   --from=builder   /app/package.json   ./package.json

# ── CASE 7: USER, EXPOSE, HEALTHCHECK ─────────────────────────────────────
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER   appuser

EXPOSE   $PORT

HEALTHCHECK   --interval=30s   --timeout=10s   --start-period=5s   --retries=3 \
    CMD   curl   -f   http://localhost:$PORT/health   ||   exit   1

# ── CASE 8: ENTRYPOINT and CMD ────────────────────────────────────────────
ENTRYPOINT   ["node"]
CMD   ["dist/server.js"]
