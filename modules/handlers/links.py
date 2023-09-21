"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot, tg_client
from modules.env import tg_user_name
from modules.utils import check_in_group, check_login, cmd_parser, delete_message
from modules.global_var import links_res


@tg_bot.on(events.NewMessage(pattern="/links", incoming=True, from_users=tg_user_name))
@check_in_group
@check_login
async def links_handler(event):
    try:
        cmd = cmd_parser(event)
        link = cmd[0]
        head_message_id = int(link.split('/')[-1])
        link_body = '/'.join(link.split('/')[:-1])
        offsets = int(cmd[1])
        await delete_message(event)
        for offset in range(offsets):
            await tg_client.send_message(event.chat_id, message='%s/%d'%(link_body, head_message_id + offset))
    except:
        await event.reply(links_res)
        raise events.StopPropagation
    raise events.StopPropagation