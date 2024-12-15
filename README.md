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
- Based on [gramme.rs](https://github.com/Lonami/grammers).
- Works only in Group.
- Transfer files you send or forward.
- Transfer restricted content.
- Transfer files from url.
- No file size limitation.
- Doesn't occupy local space, works entirely on memory through multipart transfer.
- Support multiple OneDrive accounts.
- Support OneDrive directory changing.
- Support multitasking in parallel.
- Support task cancellation.

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
            - /path/to/*.crt:/ssl/server.crt
            - /path/to/*.key:/ssl/server.key
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
        - Explain: The authorization code will be sent through the uri you offer, like `https://example.com/auth?code=xxxxxxx`.
    - Press `Register`.
    - In application's `Overview`, record `Application (client) ID` as `od_client_id`.
    - Go to application's `Certificates & secrets`, press `Client secrets`, and press `New client secret`. Then fill `Description`, and choose an `Expires`. Finnaly, press `Add`. Record `Value` as `od_client_secret`.
10. `od_root_path` is a directory on OneDrive. Like `/Videos/from-telegram`. Default to `/`.
11. `auto_delete` decides whether bot can auto delete message. Pass `true` or `false`. Optional, default to `false`.

### Dev environment
You don't have to read this section if you don't want to debug.

1. `port` is the port of the authorization server, default to `8080`.
2. `trace_level` defines the tracing level of the log, default to `info`.
3. `worker_num` controls the the maximum number of parallel tasks, default to `5`.

## Usage
### Before Start (Important!)
- Create a group.
- In bot's profile, press `Add to Group or Channel`.
- Add this bot to your group.
- Set this bot as Admin, and give it all rights like this  
    <img width="330" alt="image" src="https://github.com/hlf20010508/telegram-onedrive/assets/76218469/d5fc1130-493e-47fb-9c45-67c328470692">

If you don't follow these steps, the bot may not work.

### Authorization Steps
- Send `/auth`.
- Wait and you'll receive the login code from telegram.
- Visit the uri the bot sends, and submit the code.
- After submission, it will send the authorization uri for OneDrive. Visit, login and authorize.
- If the bot says `Onedrive authorization successful!`, everything is done.

### Start
- In the group, forward or upload files (or videos, photos, gifs, stickers, voices).
- If you want to transfer restricted content from a group or channel, right click the content, copy the message link, and send the link.
- Wait until the transfer completes. You can check the progress status on the latest message from the bot.
- Use `/help` for more information about other command.

## Bot Command
- `/start` to start with bot.
- `/auth` to authorize telegram and onedrive.
- `/clear` to clear history.
- `/autoDelete` to toggle whether bot should auto delete message.
- `/drive` to list all OneDrive accounts.
- `/drive add` to add a OneDrive account.
- `/drive $index` to change the OneDrive account.
- `/drive logout` to logout current OneDrive account.
- `/drive logout $index` to logout specified OneDrive account.
- `/links $message_link $range` to transfer sequential restricted content.
- `/url $file_url` to upload the file through url.
- `/logs` to send log file.
- `/logs clear` to clear logs.
- `/dir` to show current OneDrive directory.
- `/dir $path` to set OneDrive directory.
- `/dir temp $path` to set temporary OneDrive directory.
- `/dir temp cancel` to restore OneDrive directory to the previous one.
- `/dir reset` to reset OneDrive directory to default.
- `/version` to show the version.
- `/help` for help.

The bot support files with extension `.t2o` as scripts. You can use them to automate the bot.

### Example
- `/links https://t.me/c/xxxxxxx/100 2` will transfer `https://t.me/c/xxxxxxx/100` and `https://t.me/c/xxxxxxx/101`.
- `/url https://example.com/file.txt` will upload `file.txt`. The headers of the file response must includes `Content-Length`.
- In a file named `example.t2o`, write these lines for example:
    ```
    https://t.me/xxxx/100
    /links https://t.me/yyyy/200 2
    /autoDelete
    /dir temp /files
    /url https://example.com/file.txt
    ```

## Launch Through Docker
Launch
```sh
sudo docker compose up -d
```

## Links
- [Docker](https://hub.docker.com/repository/docker/hlf01/telegram-onedrive)