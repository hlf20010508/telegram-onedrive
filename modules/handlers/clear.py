"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_login, clear_history


@tg_bot.on(events.NewMessage(pattern="/clear", incoming=True, from_users=tg_user_name))
async def clear_handler(event):
    await check_in_group(event)
    await check_login(event)
    await clear_history(event)
    raise events.StopPropagation