FROM rust:1.84.1-bullseye AS build

WORKDIR /app

COPY . .

RUN mkdir bin

RUN --mount=type=cache,id=rust-build,target=/app/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=registry,target=/usr/local/cargo/registry \
    cargo build --target-dir /app/target \
    && cp /app/target/debug/mini_social_network /app/bin

FROM debian:bullseye

WORKDIR /app

COPY --from=build /app/bin/mini_social_network /app/bin/mini_social_network

COPY --from=build /app/static /app/static

EXPOSE 3000
CMD [ "/app/bin/mini_social_network" ]
