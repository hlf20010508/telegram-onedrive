# telegram-onedrive
> A telegram bot to transfer files to onedrive without file size limitation. Restricted content supported.

## Introduction
This project is based on telethon.

This bot works in Group or Channel.

This bot can transfer the file you send or forward to OneDrive automatically.

It can even transfer restricted content, just send the message link.

No file size limitation.

As we know, Telegram bot account can't download or upload large files.

So this project use a user account to download files and a bot account to play the role of a server.

That's why you need to prepare a lot of things to use this bot.

## Bot Command
- `/start` to start with bot.
- `/auth` to authorize telegram and onedrive.
- `/status` to show pinned status message.
- `/clear` to clear all history except status message.
- `/links message_link range` to transfer sequential restricted content.
- `/url file_url` to upload file through url.
- `/autoDelete true/false` decides whether bot can auto delete message.
- `/help` for help.

Example:  
- `/links https://t.me/c/xxxxxxx/100 2` will transfer `https://t.me/c/xxxxxxx/100` and `https://t.me/c/xxxxxxx/101`.
- `/url https://example.com/file.txt` will upload `file.txt` to Onedrive. It calls Onedrive's API, which means Onedrive's server will visit the url and download the file for you. If the url is invalid to OneDrive, the bot will try using bot's uploader to transfer.

## Authorization Steps
- Send `/auth`.
- Wait and you'll receive the login code from telegram.
- Visit the uri the bot sends, and submit the code.
- After submission, it will send the authorization uri for OneDrive. Visit, login and authorize.
- If the bot says `Onedrive authorization successful!`, everything is done.

## Usage
- Add this bot to a group or channel.
- In the group or channel, forward or upload files(or videos, photos, gifs, stickers, voices).
- If you want to transfer restricted content from a group or channel, right click the content, copy the message link, and send the link.
- Wait until the transfer completes. You can check status on pinned status message.
- Use `/help` for more information about other command.

## Preparation
1. Open `docker-compose.yml` and edit the environment config.
2. `server_uri` is your domain. You need to specify a port, like `https://example.com:8080`, or `https://127.0.0.1:8080` if you don't have a web server. Protocol must be "https", not "http".
    - The self-signed ssl keys may be expired, you can remind me for an update.
    - Some web browser may prevent you from visiting this url because of ssl mismatch. Try using [Chromium](https://download-chromium.appspot.com).
    - If you want to specify your own ssl keys, especially if you have your own site, or the self-signed ssl keys have expired, you can import your ssl keys like this:
        ```docker-compose.yml
        services:
        telegram-onedrive:
          ...
          volumes:
            - /path/to/*.crt:/telegram-onedrive/ssl/server.crt
            - /path/to/*.key:/telegram-onedrive/ssl/server.key
          ...
        ```
3. Create a Telegram bot through [BotFather](https://t.me/BotFather). Record `token` as `tg_bot_token`.
4. Create a Telegram application on [my.telegram.org](https://my.telegram.org). See [details](https://docs.telethon.dev/en/stable/basic/signing-in.html). Record `api_id` as `tg_api_id`, `api_hash` as `tg_api_hash`.
5. `tg_user_phone` is the phone number you just used to login to my.telegram.org.
6. `tg_user_name` is your telegram user name. Check your profile, find your user name, it should be like `@user`, then record `user` as `tg_user_name`. Optional, default to void. If you don't set this parameter, every one can control your bot.
7. Create a OneDrive application on [portal.azure.com](https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade) App registrations.
    - Press `New registrations`.
    - Fill `Name`.
    - In `Supported account types` choose `Personal Microsoft accounts only`.
    - In `Redirect URI`, `platform` select `Web`, uri domain should be the same with `server_uri`, route must be `/auth`.
        - Explain: The authorization code will be sent through the uri you offer, like `https://example.com:8080/auth?code=xxxxxxx`. So in this project, it use flask as a server to handle this request.
    - Press `Register`.
    - In application's `Overview`, record `Application (client) ID` as `od_client_id`.
    - Go to application's `Certificates & secrets`, press `Client secrets`, and press `New client secret`. Then fill `Description`, and choose an `Expires`. Finnaly, press `Add`. Record `Value` as `od_client_secret`.
8. `remote_root_path` is a directory on OneDrive. Like `/MyFiles/Telegram`. Default to `/`.
9. `delete_flag` decides whether bot can auto delete message. Pass `true` or `false`. Optional, default to `false`.
10. Optional, to keep sessions after recreating docker container, create a volume to store it in docker-compose.yml:
    ```docker-compose.yml
    services:
    telegram-onedrive:
      ...
      volumes:
        - telegram-onedrive-session:/telegram-onedrive/session
      ...
    volumes:
      telegram-onedrive-session:
    ```

## Launch Through Docker
```sh
# install docker-compose
sudo apt-get install docker-compose
# launch
sudo docker-compose up -d
```

build your docker image
```sh
sudo docker build -t YOUR_HOST_NAME/telegram-onedrive --no-cache .
```

## Links
- [Docker](https://hub.docker.com/repository/docker/hlf01/telegram-onedrive)
