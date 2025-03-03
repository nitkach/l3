FROM rust:1.84.1-bullseye AS build

WORKDIR /app

COPY . .

RUN mkdir bin

RUN --mount=type=cache,id=rust-build,target=/app/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=registry,target=/usr/local/cargo/registry \
    cargo build --target-dir /app/target \
    && cp /app/target/debug/t01 /app/bin

FROM debian:bullseye

WORKDIR /app

COPY --from=build /app/bin/t01 /app/bin/t01

COPY --from=build /app/templates /app/templates

EXPOSE 3000
CMD [ "/app/bin/t01" ]
