"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.global_var import start_res


@tg_bot.on(events.NewMessage(pattern="/start", incoming=True, from_users=tg_user_name))
async def start_handler(event):
    """Send a message when the command /start is issued."""
    await event.respond(start_res)
    raise events.StopPropagation