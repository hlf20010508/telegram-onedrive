"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import os
import asyncio
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login, cmd_parser
from modules.log import log_path
from modules.global_var import LOGS_LINES_PER_PAGE
from modules.res import logs_res


@tg_bot.on(events.NewMessage(pattern="/logs", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def logs_handler(event):
    if not os.path.exists(log_path):
        await event.respond("Logs not found.")
        raise events.StopPropagation
    cmd = cmd_parser(event)

    # /logs
    if len(cmd) == 1:
        await tg_bot.send_file(event.chat_id, log_path)

    elif len(cmd) == 2:
        sub_cmd = cmd[1]
        # /logs clear
        if sub_cmd == "clear":
            if os.path.exists(log_path):
                os.system("rm %s" % log_path)
                await event.respond("Logs cleared.")
            else:
                await event.respond("Logs not found.")

    else:
        await event.respond(logs_res)
    raise events.StopPropagation
