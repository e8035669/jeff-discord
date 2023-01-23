ARG BASE_IMAGE=messense/rust-musl-cross:armv7-musleabihf

FROM ${BASE_IMAGE} AS builder
ADD . ./
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/armv7-unknown-linux-musleabihf/release/jeff-discord \
    /usr/local/bin/
CMD /usr/local/bin/jeff-discord

