"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login


@tg_bot.on(events.NewMessage(pattern="/listUser", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def list_user_handler(event):
    message_str = ''
    users = onedrive.session.users
    current_user = onedrive.session.current_user
    for i in range(len(users)):
        message_str += f'{i + 1}. {users[i]}\n'
    message_str += f'\nCurrent user is {current_user}\n'
    message_str += f'\n```/changeUser index```\n To change current user.'
    await event.respond(message_str)
    raise events.StopPropagation