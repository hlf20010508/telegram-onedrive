"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_login


@tg_bot.on(events.NewMessage(pattern="/autoDelete", incoming=True, from_users=tg_user_name))
@check_in_group
@check_login
async def auto_delete_handler(event):
    from modules import env
    if env.delete_flag:
        env.delete_flag = False
        await event.respond("Bot won't auto delete message.")
    else:
        env.delete_flag = True
        await event.respond("Bot will auto delete message.")
    raise events.StopPropagation