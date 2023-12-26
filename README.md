# telegram-onedrive
A Telegram Bot to transfer files to OneDrive.

## Attention
- **Please read [Preparation](#preparation) carefully and don't omit any steps.**
- **Please read [Usage - Before Start](#before-start-important), or the bot may not work.**

## Account Types
### Supported
- Persoanl account.
- All types of business accounts, [details](https://learn.microsoft.com/en-us/office365/servicedescriptions/office-365-platform-service-description/office-365-platform-service-description#feature-availability-across-some-plans).
- All types of educational accounts if domain administrator exists.

### Not Supported
- All types of educational accounts if domain administrator **doesn't** exist.

### Not Supported Yet
- Microsoft 365 operated by 21Vianet(世纪互联).

## Introductions
- Based on telethon.
- Works only in Group.
- Transfer files you send or forward.
- Transfer restricted content.
- Transfer files from url.
- No file size limitation.
- Doesn't occupy local space, works entirely on memory through multipart transfer.

## Demos
<details>
    <summary>/start</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/edd3f162-02fd-43c0-a6eb-46a7df890c0d" alt="/start">
</details>
<details>
    <summary>/help</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/821053dc-5983-431d-ae83-66d095ce2a4b" alt="/help">
</details>
<details>
    <summary>/auth Telegram</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/4f35422d-cd92-4dac-ac8e-c63ead2db2cb" alt="/auth Telegram">
</details>
<details>
    <summary>/auth OneDrive</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/7dca129d-1d0f-49d3-9a88-eb2dc16956c0" alt="/auth OneDrive">
</details>
<details>
    <summary>transfer single</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/3c064e08-8051-4f4e-9896-5fca95fa707a" alt="transfer single">
</details>
<details>
    <summary>transfer multi</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/73f44d7d-e9cc-40fc-a7b1-547d04a5a0ec" alt="transfer multi">
</details>
<details>
    <summary>link</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/647f70d3-593c-4a12-bce3-462d6ae78aa5" alt="link">
</details>
<details>
    <summary>/links message_link range</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/d862d200-9a1c-4642-88c8-610d5bddb49f" alt="/links message_link range">
</details>
<details>
    <summary>/clear</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/86485b4f-57b5-4a03-b74b-3bffd2800582" alt="/clear">
</details>
<details>
    <summary>/clearLogs</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/f3a48b2a-12dc-4543-841e-bf76349f4a34" alt="/clearLogs">
</details>
<details>
    <summary>/logs</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/db07faa8-e8a9-4c4a-ae4f-bf1c88423280" alt="/logs">
</details>
<details>
    <summary>/logs range</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/b373456e-2525-45ba-9859-9580a8f93d72" alt="/logs range">
</details>
<details>
    <summary>/autoDelete</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/ff564f9f-66b0-4296-afe4-e8e3cdf70428" alt="/autoDelete">
</details>
<details>
    <summary>/url file_url</summary>
    <img src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/95994beb-815f-4e0f-a92c-69b5ffa19862" alt="/url file_url">
</details>

## Preparation
1. Open `docker-compose.yml` and edit the environment configuration.
2. `server_uri` is your domain, like `https://example.com`, or `https://127.0.0.1:xxxx` if you don't have a web server. Protocol must be "https", not "http".
    - Some web browsers may prevent you from visiting this url because of ssl mismatch. Try using [Chromium](https://download-chromium.appspot.com).
    - If you want to specify your own ssl keys, especially if you have your own site, you can import your ssl keys like this:
        ```docker-compose.yml
        services:
        telegram-onedrive:
          ...
          volumes:
            - /path/to/*.crt:/telegram-onedrive/server/ssl/server.crt
            - /path/to/*.key:/telegram-onedrive/server/ssl/server.key
          ...
        ```
3. Reflect the port:
    ```docker-compose.yml
    services:
        telegram-onedrive:
          ...
          ports:
            - xxxx:8080
          ...
    ```
4. Optional, if you're using reverse proxy, you need to set `reverse_proxy` to `true`. Default to `false`.
    Make sure your reverse proxy use ssl, real server protocol is `http`. For example, in `Nginx`:
    ```nginx
    listen 443 ssl;
    listen [::]:443 ssl;

    server_name example.com;

    ssl_certificate path/to/public.pem;
    ssl_certificate_key path/to/private.key;

    location / {
        proxy_pass http://127.0.0.1:xxxx/;
    }
    ```
5. Create a Telegram bot through [BotFather](https://t.me/BotFather). Record `token` as `tg_bot_token`.
6. Create a Telegram application on [my.telegram.org](https://my.telegram.org). See [details](https://docs.telethon.dev/en/stable/basic/signing-in.html). Record `api_id` as `tg_api_id`, `api_hash` as `tg_api_hash`.
7. `tg_user_phone` is the phone number you just used to login to my.telegram.org. It's in international format, like `+xxyyyyyyyyyyy`.
8. Optional, if you have two-step verification enabled, set `tg_user_password` as your 2FA password.
8. `tg_user_name` is your telegram user name. Check your profile, find your user name, it should be like `@user`, then record `user` as `tg_user_name`. If you need multiple users, use `,` to split, like `user1,user2`. Optional, default to void. If you don't set this parameter, everyone can control your bot.
9. Create a OneDrive application on [portal.azure.com](https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade) App registrations.
    - Press `New registrations`.
    - Fill `Name`.
    - In `Supported account types` choose `Accounts in any organizational directory and personal Microsoft accounts`.
    - In `Redirect URI`, `platform` select `Web`, uri domain should be the same with `server_uri`, route must be `/auth`, like `https://example.com/auth`.
        - Explain: The authorization code will be sent through the uri you offer, like `https://example.com/auth?code=xxxxxxx`. So in this project, it use flask as a server to handle this request.
    - Press `Register`.
    - In application's `Overview`, record `Application (client) ID` as `od_client_id`.
    - Go to application's `Certificates & secrets`, press `Client secrets`, and press `New client secret`. Then fill `Description`, and choose an `Expires`. Finnaly, press `Add`. Record `Value` as `od_client_secret`.
10. `remote_root_path` is a directory on OneDrive. Like `/Videos/from-telegram`. Default to `/`.
11. `delete_flag` decides whether bot can auto delete message. Pass `true` or `false`. Optional, default to `false`.
12. Optional, to keep sessions after recreating docker container, create a volume to store them:
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
    All you have to do is uncomment those lines, no edit needed.

## Usage
### Before Start (Important!)
- Create a group.
- In bot's profile, press `Add to Group or Channel`.
- Add this bot to your group.
- Set this bot as Admin, and give it all rights like this  
    <img width="330" alt="image" src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/d5fc1130-493e-47fb-9c45-67c328470692">

If you don't follow these steps, the bot may not works.

### Authorization Steps
- Send `/auth`.
- Wait and you'll receive the login code from telegram.
- Visit the uri the bot sends, and submit the code.
- After submission, it will send the authorization uri for OneDrive. Visit, login and authorize.
- If the bot says `Onedrive authorization successful!`, everything is done.

### Start
- In the group, forward or upload files (or videos, photos, gifs, stickers, voices).
- If you want to transfer restricted content from a group or channel, right click the content, copy the message link, and send the link.
- Wait until the transfer completes. You can check status on replied message, tap `Status` to locate current job.
- Use `/help` for more information about other command.

## Bot Command
- `/start` to start with bot.
- `/auth` to authorize telegram and onedrive.
- `/clear` to clear history.
- `/autoDelete` to toggle whether bot should auto delete message.
- `/clearLogs` to clear logs.
- `/logs` to show all logs.
- `/logout` to logout OneDrive.
- `/links message_link range` to transfer sequential restricted content.
- `/url file_url` to upload the file through url.
- `/logs range` to show the most recent logs for the specified page number.
- `/help` for help.

Example:  
- `/links https://t.me/c/xxxxxxx/100 2` will transfer `https://t.me/c/xxxxxxx/100` and `https://t.me/c/xxxxxxx/101`.
- `/url https://example.com/file.txt` will upload `file.txt` to Onedrive. It calls Onedrive's API, which means Onedrive's server will visit the url and download the file for you. If the url is invalid to OneDrive, the bot will try using bot's uploader to transfer.
- `/logs 2` will show 2 pages of the most recent logs. Each page contains 50 lines of logs.

## Launch Through Docker
Install docker compose
```sh
sudo apt-get install docker-compose-plugin
```

Launch
```sh
sudo docker compose up -d
```

Build your docker image
```sh
sudo docker build -t YOUR_HOST_NAME/telegram-onedrive --no-cache .
```

## Links
- [Docker](https://hub.docker.com/repository/docker/hlf01/telegram-onedrive)
