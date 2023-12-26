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


@tg_bot.on(events.NewMessage(pattern="/logout", incoming=True, from_users=tg_user_name))
@check_in_group
async def logout_handler(event):
    has_other_user = onedrive.logout()
    if has_other_user:
        current_user = onedrive.session.current_user
        logout_res = f"OneDrive logout successfully.\nCurrent account is {current_user}"
    else:
        logout_res = "OneDrive logout successfully."
    await event.respond(logout_res)
    raise events.StopPropagation