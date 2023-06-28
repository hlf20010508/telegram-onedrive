# telegram-onedrive
> A telegram bot for uploading files to onedrive

No download file size limit.

## bot command
- `/start` to start with bot
- `/auth`  to authorize telegram and onedrive
- `/help`  for help

## launch through docker
```sh
# install docker-compose
sudo apt-get install docker-compose
# modify environment args in docker-compose.yml
vim docker-compose.yml
# launch
sudo docker-compose up -d
```

build your docker image
```sh
sudo docker build -t YOUR_HOST_NAME/telegram-onedrive --no-cache .
```
