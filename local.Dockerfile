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
COPY suwen-ui/ ./
RUN bun --bun run build

FROM debian:trixie-slim
WORKDIR /app
RUN apt update && apt install -y wget && \
    wget -c https://github.com/nicolas-van/multirun/releases/download/1.1.3/multirun-x86_64-linux-gnu-1.1.3.tar.gz -O - | tar -xz && \
    mv multirun /bin && \
    apt remove -y wget && apt autoremove -y && apt clean && rm -rf /var/lib/apt/lists/*
ENV LANG=zh_CN.UTF-8 \
    TZ=Asia/Shanghai \
    PORT=5173 \
    FRONTEND_PORT=5173 \
    HOME=/app
COPY --from=rust-builder /app/suwen-backend suwen-backend
COPY --from=bun-builder /app/build  suwen-frontend
COPY --from=bun-builder /usr/local/bin/bun bun
RUN chmod -R 777 /app
EXPOSE 3000
CMD ["/usr/bin/multirun", "--", "/app/bun /app/suwen-frontend/index.js", "/app/suwen-backend"]

