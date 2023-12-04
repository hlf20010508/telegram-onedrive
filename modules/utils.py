"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from telethon.tl import types
import re
from urllib.parse import unquote, urlparse, parse_qs
import requests
import time
from copy import copy
from modules.client import tg_bot, tg_client, onedrive
from modules.log import logger
from modules.global_var import check_in_group_res, tg_not_login_res, od_not_login_res, file_param_name_list, base_headers
from modules.mime import mime_dict


class Status_Message:
    @classmethod
    async def create(cls, event):
        self = Status_Message()
        self.event = event
        try:
            self.msg_link = '[Status:](https://t.me/c/%d/%d)' % (self.event.message.peer_id.channel_id, self.event.message.id)
        except Exception as e:
            logger(e)
            self.msg_link = '[Status:](https://t.me/c/%d/%d)' % (self.event.message.peer_id.chat_id, self.event.message.id)
        self.status = 'In progress...'
        self.template = "Uploaded %.2fMB out of %.2fMB: %.2f%%"
        self.template_short = "Uploaded: %.2f%%"
        self.error_template = '- Error:\n%s'
        self.error_template_full = '- Error:\n%s\n\n- Progress url:\n%s\n\n- Response:\n%s'
        self.message = await self.event.respond(self.response)
        return self
    
    def __call__(self):
        return self.message
    
    @property
    def response(self):
        return '%s\n%s' % (self.msg_link, self.status)
    
    async def update(self):
        await edit_message(tg_bot, self.message, self.response)

    async def report_error(self, error, progress_url=None, response=None):
        if progress_url:
            await self.event.reply(self.error_template_full % (logger(error), progress_url, logger(response)))
        else:
            await self.event.reply(self.error_template % logger(error))
    
    async def finish(self):
        self.status = 'Done.'
        await edit_message(tg_bot, self.message, self.response)
        await delete_message(self.event)
        await delete_message(self.message)


class Callback:
    def __init__(self, event, status_message):
        self.event = event
        self.status_message = status_message
    
    async def __call__(self, current, total):
        current = current / (1024 * 1024)
        total = total / (1024 * 1024)
        self.status_message.status = self.status_message.template % (current, total, current / total * 100)
        logger(self.status_message.status)
        await self.status_message.update()


def cmd_parser(event):
    return event.text.split()


async def delete_message(message):
    from modules import env
    if env.delete_flag:
        try:
            await message.delete()
        except Exception as e:
            await message.reply(logger(e))
            await message.reply('Please set this bot as Admin, and give it ability to Delete Messages.')


# if message is not edited, it will raise MessageNotModifiedError
async def edit_message(bot, event, message):
    try:
        await bot.edit_message(event, message)
    except:
        pass


def check_in_group(func):
    async def wrapper(event, *args, **kwargs):
        if isinstance(event.message.peer_id, types.PeerUser):
            await event.respond(check_in_group_res)
            raise events.StopPropagation
        return await func(event, *args, **kwargs)
    return wrapper


def check_tg_login(func):
    async def wrapper(event, *args, **kwargs):
        try:
            if not await tg_client.get_me():
                await res_not_login(event, tg_not_login_res)
        except:
            await res_not_login(event, tg_not_login_res)
        return await func(event, *args, **kwargs)
    return wrapper


def check_od_login(func):
    async def wrapper(event, *args, **kwargs):
        if not onedrive.session:
            await res_not_login(event, od_not_login_res)
        return await func(event, *args, **kwargs)
    return wrapper


async def res_not_login(event, not_login_res):
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
    name = None
    parsed_url = urlparse(url)
    captured_value_dict = parse_qs(parsed_url.query)
    for item_name in captured_value_dict:
        if item_name.lower() in file_param_name_list:
            name = captured_value_dict[item_name]
            break
    if not name:
        name = unquote(url.split('/')[-1].split('?')[0].strip().strip("'").strip('"'))
    if name:
        return name
    else:
        return None


def get_filename(url):
    headers = copy(base_headers)
    # some video resource websites need Referer, or it will return 404
    headers['Referer'] = url
    headers["Connection"] = "close"
    response = requests.get(url, stream=True, verify=False, headers=headers)
    if response.status_code == 200:
        name = get_filename_from_cd(response.headers.get('Content-Disposition'))
        content_type = response.headers['Content-Type']
        ext = get_ext(content_type)
        if not name:
            name = get_filename_from_url(url)
        if not name:
            name = str(int(time.time()))
            if ext and content_type != 'application/octet-stream':
                name = name + ext[0]
        else:
            if ext and content_type != 'application/octet-stream':
                ori_ext = '.' + name.split('.')[-1].lower()
                if len(name) > 100:
                    name = str(int(time.time()))
                    if ori_ext in ext:
                        name = name + ori_ext
                    else:
                        name = name + ext[0]
                else:
                    if ori_ext not in ext:
                        name = name + ext[0]
            else:
                if len(name) > 100:
                    name = str(int(time.time()))
        return name, response
    raise Exception("File from url not found")


def get_ext(content_type):
    try:
        ext = mime_dict[content_type]
        return ext
    except:
        content_type_list = re.findall('([^;]+);', content_type)
        if content_type_list:
            try:
                ext = mime_dict[content_type_list[0]]
                return ext
            except:
                return None
        else:
            return None
        

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
