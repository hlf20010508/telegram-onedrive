FROM python:3.8.16-alpine3.17 AS cryptg_builder
RUN apk add --update --no-cache rustup build-base &&\
    rustup-init -y &&\
    source $HOME/.cargo/env &&\
    pip install cryptg==0.4.0

FROM python:3.8.16-alpine3.17
WORKDIR /telegram-onedrive
COPY ./ ./
RUN apk add --update --no-cache libgcc &&\
    pip install --no-cache-dir telethon requests flask onedrivesdk==1.1.8
COPY --from=cryptg_builder /usr/local/lib/python3.8/site-packages/cryptg /usr/local/lib/python3.8/site-packages
