ARG BASE_IMAGE=messense/rust-musl-cross:armv7-musleabihf

FROM ${BASE_IMAGE} AS builder
ADD Cargo.toml ./Cargo.toml
ADD Cargo.lock ./Cargo.lock
ADD src ./src
ADD migration ./migration
RUN cargo build --release
RUN musl-strip target/armv7-unknown-linux-musleabihf/release/jeff-discord

# FROM alpine:latest
FROM scratch
# RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/armv7-unknown-linux-musleabihf/release/jeff-discord \
    /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/jeff-discord"]

