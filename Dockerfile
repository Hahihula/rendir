FROM alpine:3.19 AS builder

RUN apk add --no-cache rustup cargo

RUN rustup default stable

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
COPY rustpress ./rustpress
COPY rustpress-core ./rustpress-core

RUN cargo build --release --package rustpress

FROM alpine:3.19

RUN apk add --no-cache ca-certificates

COPY --from=builder /build/target/release/rustpress /usr/local/bin/rustpress

ENTRYPOINT ["/usr/local/bin/rustpress"]