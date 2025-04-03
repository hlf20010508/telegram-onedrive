FROM rust:1.85.1-alpine3.20 AS rust-builder
WORKDIR /telegram-onedrive
COPY ./ ./
RUN apk add --update --no-cache build-base pkgconfig libressl-dev &&\
    cargo build --release

FROM scratch
COPY --from=rust-builder /telegram-onedrive/target/release/telegram-onedrive /
COPY --from=rust-builder /etc/ssl/cert.pem /etc/ssl/
COPY index.html /index.html
ENV RUST_BACKTRACE=1
ENTRYPOINT [ "/telegram-onedrive" ]
