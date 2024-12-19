FROM rust:1.83-bookworm as build

WORKDIR /app

COPY . .

RUN mkdir bin

RUN --mount=type=cache,id=rust-build,target=/app/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    cargo build --target-dir /app/target \
    && cp /app/target/debug/t07 /app/bin

FROM debian:bookworm

WORKDIR /app

COPY --from=build /app/bin/t07 /app/bin/t07

CMD ["/app/bin/t07"]
