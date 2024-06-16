"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
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
    get_filename,
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
        url = url.strip().strip("*")
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
            total_length = int(local_response.headers["Content-Length"])
        else:
            logger(local_response.headers)

            raise Exception(
                f"Content-Length not found in response headers.\nStatus code:\n{local_response.status_code}\nResponse headers:\n{local_response.headers}"
            )
        logger("upload url: %s" % url)
    except Exception as e:
        await event.reply(logger(e))
        raise events.StopPropagation

    last_remote_root_path = onedrive.remote_root_path

    try:
        callback = Callback(event, status_message)
        response_dict = await multi_parts_uploader_from_url(
            name, local_response, callback
        )
        await status_message.finish(
            path=os.path.join(last_remote_root_path, response_dict["name"]),
            size=total_length,
        )
    except Exception as e:
        logger(e)
        await status_message.report_error(e)

    onedrive.check_dir_temp()

    raise events.StopPropagation
