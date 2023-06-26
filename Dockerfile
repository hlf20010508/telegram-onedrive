FROM python:3.8.16-alpine3.17
WORKDIR /telegram-onedrive
COPY ./ ./

RUN pip install --no-cache-dir python-telegram-bot sanic onedrivesdk==1.1.8
