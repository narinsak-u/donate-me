# ---- Stage 1: build frontend (bun) ----
from oven/bun:1 AS web-build
WORKDIR /app/web
COPY web/package.json web/bun.lock ./
RUN bun install --frozen-lockfile
COPY web/ ./
RUN bun run build

# ---- Stage 2: build backend (rust) ----
FROM rust:1-slim AS server-build
WORKDIR /app/server
# cache deps เป็น layer แยก
COPY server/Cargo.toml server/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY server/ ./
# แตะไฟล์ใหม่เพื่อ rebuild จริง (ทรุด trick มาตรฐานของ Rust docker)
RUN touch src/main.rs && cargo build --release

# ---- Stage 3: runtime เล็ก ๆ ที่มี binary + static + migrations ----
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=server-build /app/server/target/release/donate-me-api ./donate-me-api
COPY --from=server-build /app/server/migrations ./migrations
COPY --from=web-build /app/web/dist ./web/dist

ENV DATABASE_URL=sqlite://data/donations.db?mode=rwc \
    RUST_LOG=info
RUN mkdir -p data uploads
EXPOSE 3000
CMD ["./donate-me-api"]
