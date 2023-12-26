"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import subprocess
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login
from modules.handlers.auth import od_auth


@tg_bot.on(events.NewMessage(pattern="/addDrive", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
async def add_drive_handler(event):
    auth_server = subprocess.Popen(('python', 'server/auth_server.py'))
    async with tg_bot.conversation(event.chat_id) as conv:
        await od_auth(conv)
    auth_server.kill()
    raise events.StopPropagation