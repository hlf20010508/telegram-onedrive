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
import re
import inspect
from telethon import TelegramClient, events
from telethon.tl import types
import requests
from onedrive import Onedrive
from log import logger

urllib3.disable_warnings()

temp_dir = "temp"
status_bar = None

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


@tg_bot.on(events.NewMessage(pattern="/start", incoming=True))
async def start(event):
    """Send a message when the command /start is issued."""
    await event.respond('''
Transfer files to Onedrive.

Forward or upload files to me, or pass message link to transfer restricted content from group or channel.

/auth: Authorize for Telegram and OneDrive.
/links message_link range: Transfer sequential restricted content.
/help: Ask for help.
    ''')
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/help", incoming=True))
async def help(event):
    """Send a message when the command /help is issued."""
    await event.respond('''
/auth to authorize for Telegram and OneDrive.
/links message_link range: Transfer sequential restricted content.

To transfer files, forward or upload to me.
To transfer restricted content, right click the content, copy the message link, and send to me.
    ''')
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/auth", incoming=True))
async def auth(event):
    if isinstance(event.message.peer_id, types.PeerUser):
        await event.respond('''
This bot must be used in a Group or Channel!

Add this bot to a Group or Channel as Admin, and give it ability to Delete Messages.
        ''')
        raise events.StopPropagation
    auth_server = subprocess.Popen(('python', 'auth_server.py'))
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

        async for message in tg_client.iter_messages(event.chat_id, filter=types.InputMessagesFilterPinned()):
            await tg_client.unpin_message(event.chat_id, message)

        auth_url = onedrive.get_auth_url()
        await conv.send_message(
            "Here are the authorization url of OneDrive:\n\n%s" % auth_url
        )
        code = od_code_callback()
        onedrive.auth(code)
        await conv.send_message("Authorization successful!")

        global status_bar
        status_bar = await conv.send_message("Status:\n\nNo job yet.")
        await tg_bot.pin_message(event.chat_id, status_bar)
    auth_server.kill()
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
        part_size = 1024 * 1024
        total_part_num = (
            1 if part_size >= document.size else math.ceil(document.size / part_size)
        )
        current_part_num = 0
        current_size = 0
        offset = 0
        if progress_callback:
            cor = progress_callback(current_size, document.size)
            if inspect.isawaitable(cor):
                await cor
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
                    cor = progress_callback(current_size, document.size)
                    if inspect.isawaitable(cor):
                        await cor

def get_link(string):
    regex = r"(?i)\b((?:https?://|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'\".,<>?«»“”‘’]))"
    url = re.findall(regex,string)   
    try:
        link = [x[0] for x in url][0]
        if link:
            return link
        else:
            return False
    except Exception:
        return False

@tg_bot.on(events.NewMessage(pattern="/links", incoming=True))
async def links(event):
    if isinstance(event.message.peer_id, types.PeerUser):
        await event.delete()
        await event.respond('''
This bot must be used in a Group or Channel!

Add this bot to a Group or Channel as Admin, and give it ability to Delete Messages.
        ''')
        raise events.StopPropagation
    try:
        cmd = event.text.split()
        link = cmd[1]
        head_message_id = int(link.split('/')[-1])
        link_body = '/'.join(link.split('/')[:-1])
        offsets = int(cmd[2])
        await event.delete()
        try:
            for offset in range(offsets):
                await tg_client.send_message(event.chat_id, message='%s/%d'%(link_body, head_message_id + offset))
        except:
            await event.delete()
            await event.respond('''
You haven't logined to Telegram.

Use /auth to login.
            ''')
            raise events.StopPropagation
    except:
        await event.delete()
        await event.respond('''
Command /links format wrong.

Usage: /links message_link range
        ''')
        raise events.StopPropagation
    raise events.StopPropagation

@tg_bot.on(events.NewMessage(incoming=True))
async def transfer(event):
    up_or_down = 'Downloaded'
    async def callback(current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        status = "%s %.2fMB out of %.2fMB: %.2f%%"% (up_or_down, current, total, current / total * 100)
        logger(status)
        msg_link = 'https://t.me/c/%d/%d'%(event.message.peer_id.channel_id, event.message.id)
        await tg_bot.edit_message(status_bar, 'Status:\n\n%s\n\n%s'%(msg_link, status))

    async def upload(local_path):
        nonlocal up_or_down
        up_or_down = "Uploaded"
        remote_path = await onedrive.upload(local_path, upload_status=callback)
        logger("File uploaded to", remote_path)
        for file in os.listdir(temp_dir):
            os.remove(os.path.join(temp_dir, file))
        await tg_bot.edit_message(status_bar, 'Status:\n\nNo job yet.')

    if isinstance(event.message.peer_id, types.PeerUser):
        await event.delete()
        await event.respond('''
This bot must be used in a Group or Channel!

Add this bot to a Group or Channel as Admin, and give it ability to Delete Messages.
        ''')
        raise events.StopPropagation

    if event.media and not isinstance(event.media, types.MessageMediaWebPage):
        try:
            message = await tg_client.get_messages(event.message.peer_id, ids=event.message.id)
        except:
            await event.delete()
            await event.respond('''
You haven't logined to Telegram.

Use /auth to login.
            ''')
            raise events.StopPropagation
        
        try:
            if "document" in event.media.to_dict().keys():
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
                            logger("File saved to", local_path)
                            await upload(local_path)
                            await message.delete()

            if "photo" in event.media.to_dict().keys():
                if message.media:
                    if "photo" in message.media.to_dict().keys():
                        if event.media.photo.id == message.media.photo.id:
                            name = "%d%s" % (event.media.photo.id, event.file.ext)
                            local_path = os.path.join(temp_dir, name)
                            await message.download_media(file=local_path, progress_callback=callback)
                            logger("File saved to", local_path)
                            await upload(local_path)
                            await message.delete()
        except Exception as e:
            logger(e)
    
    else:
        msg_link = get_link(event.text)
        if msg_link:
            chat = ""
            if "?single" in msg_link:
                msg_link = msg_link.split("?single")[0]
            msg_id = int(msg_link.split("/")[-1])
            if 't.me/c/' in msg_link:
                if 't.me/b/' in msg_link:
                    chat = str(msg_link.split("/")[-2])
                else:
                    chat = int('-100' + str(msg_link.split("/")[-2]))
            try:
                message = await tg_client.get_messages(chat, ids=msg_id)
            except:
                await event.delete()
                await event.respond('''
You haven't logined to Telegram.

Use /auth to login.
                ''')
                raise events.StopPropagation
            if message:
                try:
                    if "document" in message.media.to_dict().keys():
                        name = "%d%s" % (message.media.document.id, message.file.ext)
                        local_path = os.path.join(temp_dir, name)
                        await multi_parts_downloader(
                            tg_client,
                            message.media.document,
                            local_path,
                            progress_callback=callback,
                        )
                        logger("File saved to", local_path)
                        upload(local_path)
                        await event.delete()
                    if "photo" in message.media.to_dict().keys():
                        name = "%d%s" % (message.media.photo.id, message.file.ext)
                        local_path = os.path.join(temp_dir, name)
                        await message.download_media(file=local_path, progress_callback=callback)
                        logger("File saved to", local_path)
                        upload(local_path)
                        await event.delete()
                except Exception as e:
                    logger(e)
            else:
                await event.reply("Message not found.")
    raise events.StopPropagation

def main():
    tg_bot.run_until_disconnected()


if __name__ == "__main__":
    main()
