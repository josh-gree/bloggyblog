# Build Stage
FROM rust:alpine AS builder

# https://stackoverflow.com/a/75628548 - something to do with building ring...
RUN apk update && \
    apk upgrade

RUN apk add --no-cache musl-dev libcrypto3 libressl-dev libressl ca-certificates

WORKDIR /usr/src/
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new bloggyblog
WORKDIR /usr/src/bloggyblog
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Bundle Stage
FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/local/cargo/bin/main .
USER 1000
CMD ["./main", "serve"]