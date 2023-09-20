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
import subprocess
import re
import inspect
from io import BytesIO
from telethon import TelegramClient, events
from telethon.tl import types
import requests
from onedrive import Onedrive
from log import logger
from urllib.parse import unquote
import time
import mimetypes

if not os.path.exists('session'):
    os.mkdir('session')

urllib3.disable_warnings()

status_bar = None

cmd_helper = '''- /auth: Authorize for Telegram and OneDrive.
- /status: Show pinned status message.
- /clear: Clear all history except status message.

- `/links` message_link range: Transfer sequential restricted content.
- `/url` file_url: Upload file through url.
- `/autoDelete true` to auto delete message.
- `/autoDelete false` to not auto delete message.
'''

part_size = 2 * 1024 * 1024

# auth server
server_uri = os.environ["server_uri"]
# telegram api
tg_api_id = int(os.environ["tg_api_id"])
tg_api_hash = os.environ["tg_api_hash"]
tg_user_phone = os.environ["tg_user_phone"]
tg_user_name = os.environ.get("tg_user_name", None)
# telegram bot
tg_bot_token = os.environ["tg_bot_token"]
# onedrive
od_client_id = os.environ["od_client_id"]
od_client_secret = os.environ["od_client_secret"]
remote_root_path = os.environ.get("remote_root_path", "/")

delete_flag = True if os.environ.get("delete_flag", "false") == 'true' else False

# clients
tg_bot = TelegramClient("session/bot", tg_api_id, tg_api_hash, sequential_updates=True).start(
    bot_token=tg_bot_token
)

tg_client = TelegramClient("session/user", tg_api_id, tg_api_hash, sequential_updates=True)

onedrive = Onedrive(
    client_id=od_client_id,
    client_secret=od_client_secret,
    redirect_uri=os.path.join(server_uri, "auth"),
    remote_root_path=remote_root_path,
)


def cmd_parser(event):
    return event.text.split()[1:]


async def clear_history(event):
    ids = []
    async for message in tg_client.iter_messages(event.chat_id):
        ids.append(message.id)
    await tg_client.delete_messages(event.chat_id, ids)


async def delete_message(message):
    global delete_flag
    if delete_flag:
        await message.delete()


# if message is not edited, it will raise MessageNotModifiedError
async def edit_message(bot, event, message):
    try:
        await bot.edit_message(event, message)
    except:
        pass


async def check_in_group(event):
    if isinstance(event.message.peer_id, types.PeerUser):
        await event.respond('''
This bot must be used in a Group or Channel!

Add this bot to a Group or Channel as Admin, and give it ability to Delete Messages.
        ''')
        raise events.StopPropagation


async def check_login(event):
    try:
        if await tg_client.get_me():
            return True
        else:
            await res_not_login(event)
            return False
    except:
        await res_not_login(event)
        return False


async def res_not_login(event):
    await event.respond('''
You haven't logined to Telegram.
    ''')
    await auth(event, propagate=True)


async def download_part(client, input_location, offset, part_size):
        stream = client.iter_download(
            input_location, offset=offset, request_size=part_size, limit=part_size
        )
        part = await stream.__anext__()
        await stream.close()
        return part


async def multi_parts_uploader(
    client, document, name, conn_num=5, progress_callback=None
):
    input_location = types.InputDocumentFileLocation(
        id=document.id,
        access_hash=document.access_hash,
        file_reference=document.file_reference,
        thumb_size="",
    )

    upload_session = onedrive.multipart_upload_session_builder(name)
    uploader = onedrive.multipart_uploader(upload_session, document.size)

    task_list = []
    total_part_num = (
        1 if part_size >= document.size else math.ceil(document.size / part_size)
    )
    current_part_num = 0
    current_size = 0
    offset = 0
    pre_offset = 0
    if progress_callback:
        cor = progress_callback(current_size, document.size)
        if inspect.isawaitable(cor):
            await cor

    buffer = BytesIO()
    while current_part_num < total_part_num:
        task_list.append(
            asyncio.ensure_future(download_part(client, input_location, offset, part_size))
        )
        current_part_num += 1
        if current_part_num < total_part_num:
            offset += part_size
        if current_part_num % conn_num == 0 or current_part_num == total_part_num:
            for part in await asyncio.gather(*task_list):
                buffer.write(part)
                current_size += len(part)
            task_list.clear()
            buffer.seek(0)
            await onedrive.multipart_upload(uploader, buffer, pre_offset, buffer.getbuffer().nbytes)
            pre_offset = offset
            buffer = BytesIO()
            if progress_callback:
                cor = progress_callback(current_size, document.size)
                if inspect.isawaitable(cor):
                    await cor
    buffer.close()


def get_filename_from_cd(cd):
    """
    Get filename from Content-Disposition
    """
    if not cd:
        return None
    fname = re.findall('filename=(.+)', cd)
    if len(fname) == 0:
        return None
    return unquote(fname[0].strip().strip("'").strip('"'))


def get_filename_from_url(url):
    name = unquote(url.split('/')[-1].split('?')[0].strip().strip("'").strip('"'))
    if len(name) > 0:
        return name
    else:
        return None


def get_ext(content_type):
    return mimetypes.guess_extension(content_type)


async def multi_parts_uploader_from_url(url, progress_callback=None):
    response = requests.get(url, stream=True)
    if response.status_code == 200:
        total_length = int(response.headers['Content-Length'])
        name = get_filename_from_cd(response.headers.get('Content-Disposition'))
        if not name:
            name = get_filename_from_url(url)
            if name:
                ext = get_ext(response.headers['Content-Type'])
                if ext != name.split('.')[-1]:
                    name = name.split('.')[0] + ext
            else:
                name = str(int(time.time())) + ext

        upload_session = onedrive.multipart_upload_session_builder(name)
        uploader = onedrive.multipart_uploader(upload_session, total_length)

        offset = 0
        if progress_callback:
            cor = progress_callback(offset, total_length)
            if inspect.isawaitable(cor):
                await cor
        for chunk in response.iter_content(chunk_size=part_size):
            buffer = BytesIO()
            buffer.write(chunk)
            buffer.seek(0)
            await onedrive.multipart_upload(uploader, buffer, offset, buffer.getbuffer().nbytes)
            offset += buffer.getbuffer().nbytes
            if progress_callback:
                cor = progress_callback(offset, total_length)
                if inspect.isawaitable(cor):
                    await cor
        return name
    else:
        raise Exception("File from url not found")


def get_link(string):
    regex = r"(?i)\b((?:https?://|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'\".,<>?«»“”‘’]))"
    url = re.findall(regex,string)   
    try:
        link = [x[0] for x in url][0]
        if link:
            return link
        else:
            return False
    except:
        return False


@tg_bot.on(events.NewMessage(pattern="/start", incoming=True, from_users=tg_user_name))
async def start(event):
    """Send a message when the command /start is issued."""
    await event.respond('''
Transfer files to Onedrive.

Forward or upload files to me, or pass message link to transfer restricted content from group or channel.

%s
- /help: Ask for help.
    '''%cmd_helper)
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/help", incoming=True, from_users=tg_user_name))
async def help(event):
    """Send a message when the command /help is issued."""
    await event.respond('''
%s

- To transfer files, forward or upload to me.
- To transfer restricted content, right click the content, copy the message link, and send to me.
- Uploading through url will call Onedrive's API, which means Onedrive's server will visit the url and download the file for you. If the url is invalid to OneDrive, the bot will try using bot's uploader to transfer.
'''%cmd_helper)
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/auth", incoming=True, from_users=tg_user_name))
async def auth(event, propagate=False):
    await check_in_group(event)
    auth_server = subprocess.Popen(('python', 'auth_server.py'))
    async with tg_bot.conversation(event.chat_id) as conv:

        async def tg_code_callback():
            await conv.send_message(
                "Please visit %s to input your code to login to Telegram." % server_uri
            )
            res = requests.get(url=os.path.join(server_uri, "tg"), verify=False).json()
            while not res["success"]:
                await asyncio.sleep(1)
                res = requests.get(
                    url=os.path.join(server_uri, "tg"), verify=False
                ).json()
            return res["code"]

        async def od_code_callback():
            res = requests.get(
                url=os.path.join(server_uri, "auth"),
                params={"get": True},
                verify=False,
            ).json()
            while not res["success"]:
                await asyncio.sleep(1)
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

        try:
            onedrive.load_session()
        except:
            auth_url = onedrive.get_auth_url()
            await conv.send_message(
                "Here are the authorization url of OneDrive:\n\n%s" % auth_url
            )
            code = await od_code_callback()
            onedrive.auth(code)
        await conv.send_message("Onedrive authorization successful!")

        async for message in tg_client.iter_messages(event.chat_id, filter=types.InputMessagesFilterPinned()):
            await tg_client.unpin_message(event.chat_id, message)
        global status_bar
        status_bar = await conv.send_message("Status:\n\nNo job yet.")
        await tg_bot.pin_message(event.chat_id, status_bar)
    auth_server.kill()
    if not propagate:
        raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/autoDelete", incoming=True, from_users=tg_user_name))
async def auto_delete(event):
    global delete_flag
    error_message = '''
Command `/autoDelete` Usage:

`/autoDelete true` to auto delete message.
`/autoDelete false` to not auto delete message.
'''
    cmd = cmd_parser(event)
    if len(cmd) == 0:
        await event.respond(error_message)
    elif cmd[0] == 'true':
        delete_flag = True
        await event.respond('Bot will auto delete message.')
    elif cmd[0] == 'false':
        delete_flag = False
        await event.respond("Bot won't auto delete message.")
    else:
        await event.respond(error_message)
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/status", incoming=True, from_users=tg_user_name))
async def status(event):
    await check_in_group(event)
    if await check_login(event):
        global status_bar
        async for message in tg_client.iter_messages(event.chat_id, filter=types.InputMessagesFilterPinned()):
            await tg_client.unpin_message(event.chat_id, message)
        status_bar = await event.respond("Status:\n\nNo job yet.")
        await tg_bot.pin_message(event.chat_id, status_bar)
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/clear", incoming=True, from_users=tg_user_name))
async def clear(event):
    await check_in_group(event)
    await check_login(event)
    await clear_history(event)
    await status(event)
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/url", incoming=True, from_users=tg_user_name))
async def url(event):
    await check_in_group(event)
    await check_login(event)

    async def callback(current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        status = "Uploaded %.2fMB out of %.2fMB: %.2f%%"% (current, total, current / total * 100)
        logger(status)
        msg_link = 'https://t.me/c/%d/%d'%(event.message.peer_id.channel_id, event.message.id)
        await edit_message(tg_bot, status_bar, 'Status:\n\n%s\n\n%s'%(msg_link, status))

    try:
        cmd = cmd_parser(event)
        _url = cmd[0]
        # lest the url is bold
        _url = _url.strip().strip('*')
        name = unquote(_url.split('/')[-1])
    except:
        await event.reply('''
Command `/url` format wrong.

Usage: `/url` file_url
    ''')
        raise events.StopPropagation

    if not get_link(_url):
        await event.reply(logger("Please offer an HTTP url."))
        raise events.StopPropagation

    try:
        logger('upload url: %s' % _url)
        progress_url = onedrive.upload_from_url(_url)
        logger('progress url: %s' % progress_url)
    except Exception as e:
        await event.reply(logger(e))
        raise events.StopPropagation 

    try:
        response = onedrive.upload_from_url_progress(progress_url)
        progress = response.content
        while progress['status'] in ['notStarted', 'inProgress']:
            status = "Uploaded: %.2f%%" % float(progress['percentageComplete'])
            logger(status)
            msg_link = 'https://t.me/c/%d/%d'%(event.message.peer_id.channel_id, event.message.id)
            await edit_message(tg_bot, status_bar, 'Status:\n\n%s\n\n%s'%(msg_link, status))
            await asyncio.sleep(5)
            response = onedrive.upload_from_url_progress(progress_url)
            progress = response.content
        status = "Uploaded: %.2f%%" % float(progress['percentageComplete'])
        logger(status)
        if progress['status'] == 'completed':
            logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
            msg_link = 'https://t.me/c/%d/%d'%(event.message.peer_id.channel_id, event.message.id)
            await edit_message(tg_bot, status_bar, 'Status:\n\n%s\n\n%s'%(msg_link, status))
            if not delete_flag:
                await event.reply('Done.')
            await delete_message(event)
            await edit_message(tg_bot, status_bar, 'Status:\n\nNo job yet.')
        else:
            logger('use local uploader to upload from url')
            name = await multi_parts_uploader_from_url(_url, callback)
            logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
            if not delete_flag:
                await event.reply('Done.')
            await delete_message(event)
            await edit_message(tg_bot, status_bar, 'Status:\n\nNo job yet.')

    except Exception as e:
        if 'errorCode' in progress.keys():
            if progress['errorCode'] == 'ParameterIsTooLong' or progress['errorCode'] == 'NameContainsInvalidCharacters':
                # await event.reply(logger("Analysis: url too long.OneDrive API doesn't support long url."))
                try:
                    logger('use local uploader to upload from url')
                    name = await multi_parts_uploader_from_url(_url, callback)
                    logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
                    if not delete_flag:
                        await event.reply('Done.')
                    await delete_message(event)
                    await edit_message(tg_bot, status_bar, 'Status:\n\nNo job yet.')
                except Exception as e1:
                    await event.reply('Error: %s\nUpload url: %s\nProgress url: %s\n\nResponse: %s' % (logger(e1), _url, progress_url, logger(progress)))
            else:
                await event.reply('Error: %s\nUpload url: %s\nProgress url: %s\n\nResponse: %s' % (logger(e), _url, progress_url, logger(progress)))
                if progress['errorCode'] == 'Forbidden':
                    await event.reply(logger("Analysis: url protocol is not HTTP, or the url has been forbidden because of too many failed requests."))
                elif progress['errorCode'] == 'NotFound' or progress['errorCode'] == 'operationNotFound':
                    await event.reply(logger("Analysis: content not found."))
        else:
            await event.reply('Error: %s\nUpload url: %s\nProgress url: %s\n\nResponse: %s' % (logger(e), _url, progress_url, logger(progress)))

    raise events.StopPropagation


@tg_bot.on(events.NewMessage(pattern="/links", incoming=True, from_users=tg_user_name))
async def links(event):
    await check_in_group(event)
    await check_login(event)
    try:
        cmd = cmd_parser(event)
        link = cmd[0]
        head_message_id = int(link.split('/')[-1])
        link_body = '/'.join(link.split('/')[:-1])
        offsets = int(cmd[1])
        await delete_message(event)
        for offset in range(offsets):
            await tg_client.send_message(event.chat_id, message='%s/%d'%(link_body, head_message_id + offset))
    except:
        await event.reply('''
Command `/links` format wrong.

Usage: `/links` message_link range
        ''')
        raise events.StopPropagation
    raise events.StopPropagation


@tg_bot.on(events.NewMessage(incoming=True, from_users=tg_user_name))
async def transfer(event):
    await check_in_group(event)
    await check_login(event)

    async def callback(current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        status = "Uploaded %.2fMB out of %.2fMB: %.2f%%"% (current, total, current / total * 100)
        logger(status)
        msg_link = 'https://t.me/c/%d/%d'%(event.message.peer_id.channel_id, event.message.id)
        await edit_message(tg_bot, status_bar, 'Status:\n\n%s\n\n%s'%(msg_link, status))

    if event.media and not isinstance(event.media, types.MessageMediaWebPage):
        message = await tg_client.get_messages(event.message.peer_id, ids=event.message.id)
        
        try:
            if "document" in event.media.to_dict().keys():
                if message.media:
                    if "document" in message.media.to_dict().keys():
                        if event.media.document.id == message.media.document.id:
                            name = "%d%s" % (event.media.document.id, event.file.ext)
                            await multi_parts_uploader(tg_client, message.media.document, name, progress_callback=callback)
                            logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                            await delete_message(message)
                            await edit_message(tg_bot, status_bar, "Status:\n\nNo job yet.")

            if "photo" in event.media.to_dict().keys():
                if message.media:
                    if "photo" in message.media.to_dict().keys():
                        if event.media.photo.id == message.media.photo.id:
                            name = "%d%s" % (event.media.photo.id, event.file.ext)
                            buffer = await message.download_media(file=bytes, progress_callback=callback)
                            onedrive.stream_upload(buffer, name)
                            logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                            await delete_message(message)
                            await edit_message(tg_bot, status_bar, "Status:\n\nNo job yet.")
        except Exception as e:
            await event.reply('Error: %s' % logger(e))
    
    else:
        msg_link = get_link(event.text)
        if msg_link:
            try:
                chat = ""
                if "?single" in msg_link:
                    msg_link = msg_link.split("?single")[0]
                msg_id = int(msg_link.split("/")[-1])
                if 't.me/c/' in msg_link:
                    if 't.me/b/' in msg_link:
                        chat = str(msg_link.split("/")[-2])
                    else:
                        chat = int('-100' + str(msg_link.split("/")[-2]))

                message = await tg_client.get_messages(chat, ids=msg_id)
            except:
                logger('Not message link.')
                await event.reply("Please offer a message link.\n\nUse /help for available command.")
                raise events.StopPropagation

            if message:
                try:
                    if "document" in message.media.to_dict().keys():
                        name = "%d%s" % (message.media.document.id, message.file.ext)
                        await multi_parts_uploader(tg_client, message.media.document, name, progress_callback=callback)
                        logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                        await delete_message(event)
                        await edit_message(tg_bot, status_bar, "Status:\n\nNo job yet.")

                    if "photo" in message.media.to_dict().keys():
                        name = "%d%s" % (message.media.photo.id, message.file.ext)
                        buffer = await message.download_media(file=bytes, progress_callback=callback)
                        onedrive.stream_upload(buffer, name)
                        logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                        await delete_message(event)
                        await edit_message(tg_bot, status_bar, "Status:\n\nNo job yet.")

                except Exception as e:
                    await event.reply('Error: %s' % logger(e))
            else:
                await event.reply(logger("Message not found."))
        else:
            if event.text != '/auth':
                logger('Unknown command.')
                await event.reply("Use /help for available command.")
    raise events.StopPropagation


def main():
    tg_bot.run_until_disconnected()


if __name__ == "__main__":
    main()
