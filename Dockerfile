FROM rust:1-alpine3.21 AS builder

WORKDIR /build

# OpenSSL dev headers and pkg-config are required to build the
# reqwest -> openssl-sys dependency chain (used for HTTPS downloads
# of remote assets in the dev server).
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

COPY Cargo.toml Cargo.lock ./
COPY rendir ./rendir
COPY rendir-core ./rendir-core
COPY rendir-wasm ./rendir-wasm

RUN cargo build --release --package rendir

FROM alpine:3.21

RUN apk add --no-cache ca-certificates libgcc openssl

COPY --from=builder /build/target/release/rendir /usr/local/bin/rendir

ENTRYPOINT ["/usr/local/bin/rendir"]
