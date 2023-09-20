"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from urllib.parse import unquote
import asyncio
import os
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import Callback, check_in_group, check_login, edit_message, cmd_parser, get_link, delete_message
from modules.log import logger
from modules.transfer import multi_parts_uploader_from_url
from modules.global_var import url_res


@tg_bot.on(events.NewMessage(pattern="/url", incoming=True, from_users=tg_user_name))
async def url_handler(event):
    await check_in_group(event)
    await check_login(event)

    try:
        cmd = cmd_parser(event)
        _url = cmd[0]
        # lest the url is bold
        _url = _url.strip().strip('*')
        name = unquote(_url.split('/')[-1])
    except:
        await event.reply(url_res)
        raise events.StopPropagation

    if not get_link(_url):
        await event.reply(logger("Please offer an HTTP url."))
        raise events.StopPropagation

    status_message = await event.reply('In progress...', silent=True)

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
            await edit_message(tg_bot, status_message, 'Status:\n%s' % status)
            await asyncio.sleep(5)
            response = onedrive.upload_from_url_progress(progress_url)
            progress = response.content
        status = "Uploaded: %.2f%%" % float(progress['percentageComplete'])
        logger(status)
        await edit_message(tg_bot, status_message, 'Status:\n%s' % status)
        if progress['status'] == 'completed':
            logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
            await edit_message(tg_bot, status_message, 'Done.')
            await delete_message(event)
            await delete_message(status_message)
        else:
            logger('use local uploader to upload from url')
            callback = Callback(event, status_message)
            name = await multi_parts_uploader_from_url(_url, callback.run)
            logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
            await edit_message(tg_bot, status_message, 'Done.')
            await delete_message(event)
            await delete_message(status_message)

    except Exception as e:
        if 'errorCode' in progress.keys():
            if progress['errorCode'] == 'ParameterIsTooLong' or progress['errorCode'] == 'NameContainsInvalidCharacters':
                # await event.reply(logger("Analysis: url too long.OneDrive API doesn't support long url."))
                try:
                    logger('use local uploader to upload from url')
                    callback = Callback(event, status_message)
                    name = await multi_parts_uploader_from_url(_url, callback.run)
                    logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
                    await edit_message(tg_bot, status_message, 'Done.')
                    await delete_message(event)
                    await delete_message(status_message)
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