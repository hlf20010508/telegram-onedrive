# telegram-onedrive
> A telegram bot for uploading files to onedrive

Support List:
- Video

Download file limit is 20MB due to telegram bot limitation.

## bot command
- `/start` to start with bot
- `/auth`  to authorize onedrive
- `/help`  for help

## launch through docker
```sh
# install docker-compose
sudo apt-get install docker-compose
# modify environment args in docker-compose.yml
# telegram: TOKEN
# onedrive: client_id, client_secret, redirect_uri, remote_root_path
vim docker-compose.yml
# launch
sudo docker-compose up -d
```

build your docker image
```sh
sudo docker build -t YOUR_HOST_NAME/telegram-onedrive --no-cache .
```

## launch directly
```sh
# install pipenv
pip install pipenv
# install dependences
pipenv install
# set environment args
export TOKEN=$TOKEN
export client_id=$client_id
export client_secret=$client_secret
export redirect_uri=$redirect_uri
export remote_root_path=$remote_root_path
# launch
pipenv run python bot.py
```
