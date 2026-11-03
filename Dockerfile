FROM rust:1.97-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked && rm -rf src
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/dispatchly /usr/local/bin/dispatchly
COPY templates ./templates
COPY static ./static
ENV PORT=5115 \
    DISPATCHLY_DB=/data/dispatchly.sqlite3 \
    DISPATCHLY_RESEED=1
RUN mkdir -p /data
EXPOSE 5115
CMD ["dispatchly"]
