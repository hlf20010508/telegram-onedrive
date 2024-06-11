FROM hlf01/cryptg:0.4.0-python3.8.16-alpine3.17 AS cryptg_builder
FROM hlf01/cryptography:41.0.7-python3.8.16-alpine3.17 AS cryptography_builder
FROM python:3.8.16-alpine3.17

WORKDIR /telegram-onedrive

COPY ./ ./

RUN apk add --update --no-cache libgcc &&\
    pip install --no-cache-dir telethon requests flask onedrivesdk==1.1.8

COPY --from=cryptg_builder /cryptg /usr/local/lib/python3.8/site-packages/cryptg
COPY --from=cryptography_builder /packages /usr/local/lib/python3.8/site-packages
