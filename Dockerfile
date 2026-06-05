FROM rust:1-alpine3.21 AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY rustpress ./rustpress
COPY rustpress-core ./rustpress-core

RUN cargo build --release --package rustpress

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /build/target/release/rustpress /usr/local/bin/rustpress

ENTRYPOINT ["/usr/local/bin/rustpress"]
