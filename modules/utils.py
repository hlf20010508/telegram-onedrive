"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from telethon.tl import types
import re
from urllib.parse import unquote
import mimetypes
from modules.client import tg_bot, tg_client
from modules.log import logger
from modules.global_var import check_in_group_res, not_login_res

class Callback:
    def __init__(self, event, status_message):
        self.event = event
        self.status_message = status_message
    
    async def run(self, current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        status = "Uploaded %.2fMB out of %.2fMB: %.2f%%"% (current, total, current / total * 100)
        logger(status)
        await edit_message(tg_bot, self.status_message, 'Status:\n%s' % status)


def cmd_parser(event):
    return event.text.split()[1:]


async def clear_history(event):
    ids = []
    async for message in tg_client.iter_messages(event.chat_id):
        ids.append(message.id)
    await tg_client.delete_messages(event.chat_id, ids)


async def delete_message(message):
    from modules import env
    if env.delete_flag:
        await message.delete()


# if message is not edited, it will raise MessageNotModifiedError
async def edit_message(bot, event, message):
    try:
        await bot.edit_message(event, message)
    except:
        pass


async def check_in_group(event):
    if isinstance(event.message.peer_id, types.PeerUser):
        await event.respond(check_in_group_res)
        raise events.StopPropagation


async def check_login(event):
    try:
        if await tg_client.get_me():
            return True
        else:
            await res_not_login(event)
            return False
    except:
        await res_not_login(event)
        return False


async def res_not_login(event):
    from modules.handlers.auth import auth_handler
    await event.respond(not_login_res)
    await auth_handler(event, propagate=True)


def get_filename_from_cd(cd):
    """
    Get filename from Content-Disposition
    """
    if not cd:
        return None
    fname = re.findall('filename=(.+)', cd)
    if len(fname) == 0:
        return None
    return unquote(fname[0].strip().strip("'").strip('"'))


def get_filename_from_url(url):
    name = unquote(url.split('/')[-1].split('?')[0].strip().strip("'").strip('"'))
    if len(name) > 0:
        return name
    else:
        return None


def get_ext(content_type):
    return mimetypes.guess_extension(content_type)


def get_link(string):
    regex = r"(?i)\b((?:https?://|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'\".,<>?«»“”‘’]))"
    url = re.findall(regex,string)   
    try:
        link = [x[0] for x in url][0]
        if link:
            return link
        else:
            return False
    except:
        return False
