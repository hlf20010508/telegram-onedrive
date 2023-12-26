"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login, cmd_parser


@tg_bot.on(events.NewMessage(pattern="/changeDrive", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def change_drive_handler(event):
    cmd = cmd_parser(event)
    if len(cmd) == 2:
        index = int(cmd[1]) - 1
        users = onedrive.session.users
        old_user = onedrive.session.current_user
        new_user = onedrive.session.change_user(users[index])
        await event.respond(f'Changed account from\n{old_user}\nto\n{new_user}')
    raise events.StopPropagation