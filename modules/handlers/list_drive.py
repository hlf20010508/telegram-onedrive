"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login
from modules.onedrive.database import UserNotFoundException


@tg_bot.on(events.NewMessage(pattern="/listDrive", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
async def list_drive_handler(event):
    message_str = ''
    try:
        users = onedrive.session.users
        current_user = onedrive.session.current_user
    except:
        await event.respond('No account found.')
        raise events.StopPropagation

    for i in range(len(users)):
        message_str += f'{i + 1}. {users[i]}\n'
    message_str += f'\nCurrent account is {current_user}\n'
    message_str += f'\n```/changeDrive index```\n To change current account.'
    
    await event.respond(message_str)
    raise events.StopPropagation