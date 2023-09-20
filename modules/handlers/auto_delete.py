"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot
from modules.utils import cmd_parser
from modules.env import tg_user_name
from modules.global_var import auto_delete_res


@tg_bot.on(events.NewMessage(pattern="/autoDelete", incoming=True, from_users=tg_user_name))
async def auto_delete_handler(event):
    
    cmd = cmd_parser(event)
    if len(cmd) == 0:
        await event.respond(auto_delete_res)
    elif cmd[0] == 'true':
        from modules import env
        env.delete_flag = True
        await event.respond('Bot will auto delete message.')
    elif cmd[0] == 'false':
        from modules import env
        env.delete_flag = False
        await event.respond("Bot won't auto delete message.")
    else:
        await event.respond(auto_delete_res)
    raise events.StopPropagation