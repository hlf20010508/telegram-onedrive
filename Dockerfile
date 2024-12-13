FROM rust:1.80.1-alpine3.20 AS rust-builder
WORKDIR /telegram-onedrive
COPY ./ ./
RUN apk add --update --no-cache build-base pkgconfig libressl-dev &&\
    cargo build --release

FROM alpine:3.20 as certs

FROM scratch
COPY --from=rust-builder /telegram-onedrive/target/release/telegram-onedrive /
COPY --from=rust-builder /telegram-onedrive/index.html /
COPY --from=certs /etc/ssl/cert.pem /etc/ssl/
ENV RUST_BACKTRACE=1
ENTRYPOINT [ "/telegram-onedrive" ]
