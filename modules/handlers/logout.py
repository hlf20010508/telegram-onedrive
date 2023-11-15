"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import check_in_group
from modules.global_var import logout_res


@tg_bot.on(events.NewMessage(pattern="/logout", incoming=True, from_users=tg_user_name))
@check_in_group
async def logout_handler(event):
    onedrive.logout()
    await event.respond(logout_res)
    raise events.StopPropagation