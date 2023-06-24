FROM rust:1-alpine AS builder

WORKDIR /usr/src/app
COPY . .
RUN apk add --no-cache musl-dev && \
    cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /usr/src/app/target/release/yta-rs /usr/local/bin/yta-rs

CMD [ "yta-rs" ]