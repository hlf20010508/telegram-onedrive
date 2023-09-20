"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from telethon.tl import types
import os
from modules.client import tg_bot, tg_client, onedrive
from modules.env import tg_user_name, remote_root_path
from modules.utils import Callback, check_in_group, check_login, edit_message, delete_message, get_link
from modules.log import logger
from modules.transfer import multi_parts_uploader


@tg_bot.on(events.NewMessage(incoming=True, from_users=tg_user_name))
async def transfer_handler(event):
    await check_in_group(event)
    await check_login(event)

    if event.media and not isinstance(event.media, types.MessageMediaWebPage):
        message = await tg_client.get_messages(event.message.peer_id, ids=event.message.id)
        
        try:
            if "document" in event.media.to_dict().keys():
                if message.media:
                    if "document" in message.media.to_dict().keys():
                        if event.media.document.id == message.media.document.id:
                            name = "%d%s" % (event.media.document.id, event.file.ext)
                            status_message = await event.reply('In progress...', silent=True)
                            callback = Callback(event, status_message)
                            await multi_parts_uploader(tg_client, message.media.document, name, progress_callback=callback.run)
                            logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                            await edit_message(tg_bot, status_message, 'Done.')
                            await delete_message(message)
                            await delete_message(status_message)

            if "photo" in event.media.to_dict().keys():
                if message.media:
                    if "photo" in message.media.to_dict().keys():
                        if event.media.photo.id == message.media.photo.id:
                            name = "%d%s" % (event.media.photo.id, event.file.ext)
                            status_message = await event.reply('In progress...', silent=True)
                            callback = Callback(event, status_message)
                            buffer = await message.download_media(file=bytes, progress_callback=callback.run)
                            onedrive.stream_upload(buffer, name)
                            logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                            await edit_message(tg_bot, status_message, 'Done.')
                            await delete_message(message)
                            await delete_message(status_message)
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
                        status_message = await event.reply('In progress...', silent=True)
                        callback = Callback(event, status_message)
                        await multi_parts_uploader(tg_client, message.media.document, name, progress_callback=callback.run)
                        logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                        await edit_message(tg_bot, status_message, 'Done.')
                        await delete_message(event)
                        await delete_message(status_message)

                    if "photo" in message.media.to_dict().keys():
                        name = "%d%s" % (message.media.photo.id, message.file.ext)
                        status_message = await event.reply('In progress...', silent=True)
                        callback = Callback(event, status_message)
                        buffer = await message.download_media(file=bytes, progress_callback=callback.run)
                        onedrive.stream_upload(buffer, name)
                        logger("File uploaded to %s" % os.path.join(remote_root_path, name))
                        await edit_message(tg_bot, status_message, 'Done.')
                        await delete_message(event)
                        await delete_message(status_message)

                except Exception as e:
                    await event.reply('Error: %s' % logger(e))
            else:
                await event.reply(logger("Message not found."))
        else:
            if event.text != '/auth':
                logger('Unknown command.')
                await event.reply("Use /help for available command.")
    raise events.StopPropagation