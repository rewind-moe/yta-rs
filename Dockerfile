FROM rust:1-alpine AS builder

WORKDIR /usr/src/app

COPY Cargo.toml .
RUN set -ex; \
    apk add --no-cache musl-dev; \
    mkdir src; \
    echo 'fn main() {}' > src/main.rs; \
    echo 'fn lib() {}' > src/lib.rs; \
    cargo build --release; \
    rm -rf src;

COPY . .
RUN touch src/main.rs src/lib.rs && \
    cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /usr/src/app/target/release/yta-rs /usr/local/bin/yta-rs

CMD [ "yta-rs" ]