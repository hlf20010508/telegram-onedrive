FROM hlf01/cryptg:0.4.0-python3.8.16-alpine3.17 AS cryptg_builder
FROM hlf01/cryptography:41.0.7-python3.8.16-alpine3.17 AS cryptography_builder
FROM python:3.8.16-alpine3.17
RUN apk add --update --no-cache libgcc git &&\
    pip install --no-cache-dir telethon requests flask onedrivesdk==1.1.8 git+https://github.com/hlf20010508/LTorrent.git@1.6.0#subdirectory=ltorrent_async &&\
    apk del git
WORKDIR /telegram-onedrive
COPY ./ ./
COPY --from=cryptg_builder /cryptg /usr/local/lib/python3.8/site-packages/cryptg
COPY --from=cryptography_builder /packages /usr/local/lib/python3.8/site-packages
