"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, tg_client
from modules.env import tg_user_name
from modules.utils import check_in_group, check_login


async def clear_history(event):
    ids = []
    async for message in tg_client.iter_messages(event.chat_id):
        ids.append(message.id)
    await tg_client.delete_messages(event.chat_id, ids)


@tg_bot.on(events.NewMessage(pattern="/clear", incoming=True, from_users=tg_user_name))
@check_in_group
@check_login
async def clear_handler(event):
    await clear_history(event)
    raise events.StopPropagation