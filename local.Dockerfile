FROM lukemathwalker/cargo-chef:latest-rust-1.90-slim-trixie AS chef
WORKDIR /app

FROM chef AS rust-planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS rust-builder
COPY --from=rust-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release && \
    cp target/release/suwen suwen-backend

FROM oven/bun:1.3.1-slim AS bun-builder
WORKDIR /app
COPY suwen-ui/package.json suwen-ui/bun.lock ./
RUN bun install
COPY suwen-ui/* ./
RUN bun --bun run build && \
    bun build ./build/index.js --compile --outfile ./suwen-frontend

FROM debian:trixie-slim
WORKDIR /app
RUN apt update && apt install -y multirun && \
    apt clean && rm -rf /var/lib/apt/lists/*
ENV LANG=zh_CN.UTF-8 \
    TZ=Asia/Shanghai \
    PORT=5173 \
    FRONTEND_PORT=5173 \
    RUST_LOG=NONE,suwen=INFO,suwen-api=INFO \
    HOME=/app
COPY --from=rust-builder /app/suwen-backend suwen-backend
COPY --from=bun-builder /app/suwen-frontend  suwen-frontend
RUN chmod -R 777 /app
EXPOSE 3000
CMD ["/usr/bin/multirun", "--", "/app/suwen-frontend", "/app/suwen-backend"]
