"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import os
import urllib3
import asyncio
import math
from time import sleep
import subprocess
from telethon import TelegramClient, events, utils
from telethon.tl import types
import requests
from onedrive import Onedrive

auth_server = subprocess.Popen(('python', 'auth_server.py'))

urllib3.disable_warnings()

temp_dir = "temp"

# auth server
server_uri = os.environ["server_uri"]

# telegram api
tg_api_id = int(os.environ["tg_api_id"])
tg_api_hash = os.environ["tg_api_hash"]
tg_user_phone = os.environ["tg_user_phone"]

# telegram bot
tg_bot_token = os.environ["tg_bot_token"]

# onedrive
od_client_id = os.environ["od_client_id"]
od_client_secret = os.environ["od_client_secret"]
remote_root_path = os.environ.get("remote_root_path", "/")

# clients
tg_bot = TelegramClient("bot", tg_api_id, tg_api_hash, sequential_updates=True).start(
    bot_token=tg_bot_token
)
tg_client = TelegramClient("user", tg_api_id, tg_api_hash, sequential_updates=True)

onedrive = Onedrive(
    client_id=od_client_id,
    client_secret=od_client_secret,
    redirect_uri=os.path.join(server_uri, "auth"),
    remote_root_path=remote_root_path,
)

if not os.path.exists(temp_dir):
    os.mkdir(temp_dir)
else:
    for file in os.listdir(temp_dir):
        os.remove(os.path.join(temp_dir, file))


@tg_bot.on(events.NewMessage(pattern="/start"))
async def start(event):
    """Send a message when the command /start is issued."""
    await event.respond(
        "Upload files to Onedrive.\n`/auth` to authorize for Telegram and OneDrive.\n`/help` for help."
    )
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/help"))
async def help(event):
    """Send a message when the command /help is issued."""
    await event.respond("`/auth` to authorize for Telegram and OneDrive.")
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/auth"))
async def auth(event):
    async with tg_bot.conversation(event.chat_id) as conv:

        async def tg_code_callback():
            await conv.send_message(
                "Please visit %s to input your code to login to Telegram." % server_uri
            )
            res = requests.get(url=os.path.join(server_uri, "tg"), verify=False).json()
            while not res["success"]:
                sleep(1)
                res = requests.get(
                    url=os.path.join(server_uri, "tg"), verify=False
                ).json()
            return res["code"]

        def od_code_callback():
            res = requests.get(
                url=os.path.join(server_uri, "auth"),
                params={"get": True},
                verify=False,
            ).json()
            while not res["success"]:
                sleep(1)
                res = requests.get(
                    url=os.path.join(server_uri, "auth"),
                    params={"get": True},
                    verify=False,
                ).json()
            return res["code"]

        await conv.send_message("Logining into Telegram...")
        global tg_client
        tg_client = await tg_client.start(tg_user_phone, code_callback=tg_code_callback)
        await conv.send_message("Login to Telegram successful!")
        auth_url = onedrive.get_auth_url()
        await conv.send_message(
            "Here are the authorization url of OneDrive:\n\n%s" % auth_url
        )
        code = od_code_callback()
        onedrive.auth(code)
        await conv.send_message("Authorization successful!")
    raise events.StopPropagation


async def multi_parts_downloader(
    client, document, path, conn_num=10, progress_callback=None
):
    async def download_part(input_location, offset, part_size):
        stream = client.iter_download(
            input_location, offset=offset, request_size=part_size, limit=part_size
        )
        part = await stream.__anext__()
        await stream.close()
        return part

    with open(path, "wb") as file:
        input_location = types.InputDocumentFileLocation(
            id=document.id,
            access_hash=document.access_hash,
            file_reference=document.file_reference,
            thumb_size="",
        )
        task_list = []
        part_size = int(utils.get_appropriated_part_size(document.size) * 1024)
        total_part_num = (
            1 if part_size >= document.size else math.ceil(document.size / part_size)
        )
        current_part_num = 0
        current_size = 0
        offset = 0
        while current_part_num < total_part_num:
            task_list.append(
                asyncio.ensure_future(download_part(input_location, offset, part_size))
            )
            current_part_num += 1
            if current_part_num < total_part_num:
                offset += part_size
            if current_part_num % conn_num == 0 or current_part_num == total_part_num:
                for part in await asyncio.gather(*task_list):
                    file.write(part)
                    current_size += len(part)
                task_list.clear()
                if progress_callback:
                    progress_callback(current_size, document.size)


@tg_bot.on(events.NewMessage)
async def transfer(event):
    def callback(current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        print(
            "Downloaded %.2fMB out of %.2fMB: %.2f%%"
            % (current, total, current / total * 100)
        )

    def upload(local_path):
        remote_path = onedrive.upload(local_path, show_status=True)
        print("File uploaded to", remote_path)
        for file in os.listdir(temp_dir):
            os.remove(os.path.join(temp_dir, file))

    if event.media:
        onedrive_bot = await tg_bot.get_me()
        onedrive_bot = await tg_client.get_entity("@%s" % onedrive_bot.username)
        iter_messages = tg_client.iter_messages(onedrive_bot)
        if "document" in event.media.to_dict().keys():
            async for message in iter_messages:
                if message.media:
                    if "document" in message.media.to_dict().keys():
                        if event.media.document.id == message.media.document.id:
                            name = "%d%s" % (event.media.document.id, event.file.ext)
                            local_path = os.path.join(temp_dir, name)
                            await multi_parts_downloader(
                                tg_client,
                                message.media.document,
                                local_path,
                                progress_callback=callback,
                            )
                            print("File saved to", local_path)
                            upload(local_path)
                            await message.delete()
                            break

        if "photo" in event.media.to_dict().keys():
            async for message in iter_messages:
                if message.media:
                    if "photo" in message.media.to_dict().keys():
                        if event.media.photo.id == message.media.photo.id:
                            name = "%d%s" % (event.media.photo.id, event.file.ext)
                            local_path = os.path.join(temp_dir, name)
                            await message.download_media(file=local_path)
                            print("File saved to", local_path)
                            upload(local_path)
                            await message.delete()
                            break


def main():
    tg_bot.run_until_disconnected()


if __name__ == "__main__":
    main()
