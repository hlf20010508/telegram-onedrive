"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import asyncio
import os
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import (
    Callback,
    Status_Message,
    check_in_group,
    check_tg_login,
    check_od_login,
    cmd_parser,
    get_link,
    get_filename
)
from modules.log import logger
from modules.transfer import multi_parts_uploader_from_url
from modules.res import url_res


@tg_bot.on(events.NewMessage(pattern="/url", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def url_handler(event):
    try:
        cmd = cmd_parser(event)
        url = cmd[1]
        # lest the url is bold
        url = url.strip().strip('*')
    except:
        await event.reply(url_res)
        raise events.StopPropagation

    if not get_link(url):
        await event.reply(logger("Please offer an HTTP url."))
        raise events.StopPropagation

    status_message = await Status_Message.create(event)

    try:
        name, local_response = get_filename(url)
        if "Content-Length" in local_response.headers:
            total_length = int(local_response.headers['Content-Length']) / (1024 * 1024)
        elif "Transfer-Encoding" in local_response.headers and local_response.headers["Transfer-Encoding"] == "chunked":
            total_length = -1
        else:
            raise Exception(
                "Neither Content-Length nor Transfer-Encoding is in response headers.\nStatus code:\n%s\nResponse:\n%s" %
                (local_response.status_code, local_response.headers)
            )
        logger('upload url: %s' % url)
    except Exception as e:
        await event.reply(logger(e))
        raise events.StopPropagation 

    last_remote_root_path = onedrive.remote_root_path
    try:
        progress_url = onedrive.upload_from_url(url, name)
        logger('progress url: %s' % progress_url)
        while True:
            response = onedrive.upload_from_url_progress(progress_url)
            progress = response.content
            if progress['status'] in ['notStarted', 'inProgress', 'completed', 'waiting']:
                percentage = float(progress['percentageComplete'])
                if total_length > 0:
                    status_message.status = status_message.template % (total_length * percentage / 100, total_length, percentage)
                else:
                    status_message.status = status_message.template_short % percentage
                logger(status_message.status)
                await status_message.update()

                if progress['status'] == 'completed':
                    await status_message.finish(
                        path=os.path.join(last_remote_root_path, name),
                        size=total_length
                    )
                    break

                await asyncio.sleep(5)
            else:
                raise Exception('status error: %s' % progress)

    except Exception as e:
        logger(e)
        try:
            if total_length > 0:
                logger('use local uploader to upload from url')
                callback = Callback(event, status_message)
                await multi_parts_uploader_from_url(name, local_response, callback)
                await status_message.finish(
                    path=os.path.join(last_remote_root_path, name),
                    size=total_length
                )
            else:
                logger(local_response.headers)
                # this happends when downloading github release assets
                # sometimes it has Content-Length, sometimes not
                await status_message.report_error("Content-Length not found in response headers.")
        except Exception as e:
            try:
                await status_message.report_error(e, progress_url, progress)
            except Exception as e1:
                logger(e1)
                await status_message.report_error(e)
    onedrive.check_dir_temp()
    raise events.StopPropagation
