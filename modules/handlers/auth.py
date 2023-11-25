"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import subprocess
import requests
import asyncio
import os
from telethon import events
from telethon.errors.rpcerrorlist import FloodWaitError
from modules.env import tg_user_name, tg_user_phone, server_uri
from modules.utils import check_in_group
from modules.client import tg_bot, tg_client, onedrive, init_tg_client
from modules.log import logger
from modules.global_var import tg_login_max_attempts


class Code_Callback:
    def __init__(self, conv, type):
        self.conv = conv
        self.type = type
        self.tg_login_attempts = 0
        self.tg_code_resent = False

    async def tg_code(self):
        if self.tg_code_resent:
            if self.tg_login_attempts > 0:
                await self.conv.send_message(
                    logger('Invalid code. Please try again.')
                )
            else:
                await self.conv.send_message(logger("New code sent, please try again."))
        else:
            if self.tg_login_attempts > 0:
                await self.conv.send_message(
                    logger('Invalid code. Please try again.')
                )
            else:
                await self.conv.send_message(
                    logger("Please visit %s to input your code to login to Telegram." % server_uri)
                )
        self.tg_login_attempts += 1
        while True:
            try:
                requests.get(
                    url=os.path.join(server_uri, "tg"),
                    params={"refresh": True},
                    verify=False
                )
                break
            except:
                pass
        
        while True:
            try:
                res = requests.get(
                    url=os.path.join(server_uri, "tg"),
                    verify=False
                ).json()
                if res["success"]:
                    return res["code"]
                await asyncio.sleep(1)
            except:
                pass
    
    async def od_code(self):
        auth_url = onedrive.get_auth_url()
        await self.conv.send_message(
            logger("Here are the authorization url of OneDrive:\n\n%s" % auth_url)
        )
        while True:
            try:
                res = requests.get(
                    url=os.path.join(server_uri, "auth"),
                    params={"get": True},
                    verify=False,
                ).json()
                if res["success"]:
                    return res["code"]
                elif res["failed"]:
                    await self.conv.send_message(
                        logger(res['failed_info'])
                    )
                    return False
                await asyncio.sleep(1)
            except:
                pass
        
    async def code(self):
        if self.type == "tg":
            return await self.tg_code()
        elif self.type == "od":
            return await self.od_code()

    async def __call__(self):
        return await self.code()


@tg_bot.on(events.NewMessage(pattern="/auth", incoming=True, from_users=tg_user_name))
@check_in_group
async def auth_handler(event, propagate=False):
    auth_server = subprocess.Popen(('python', 'server/auth_server.py'))
    async with tg_bot.conversation(event.chat_id) as conv:
        tg_code_callback = Code_Callback(conv, 'tg')
        od_code_callback = Code_Callback(conv, 'od')
        global tg_client
        while True:
            try:
                await conv.send_message(logger("Logining into Telegram..."))
                _tg_client = await tg_client.start(tg_user_phone, code_callback=tg_code_callback, max_attempts=tg_login_max_attempts)
                tg_client = _tg_client
                await conv.send_message(logger("Login to Telegram successful!"))
                break
            except RuntimeError as e:
                await tg_client.log_out()
                tg_client = init_tg_client()
                tg_code_callback.tg_login_attempts = 0
                tg_code_callback.tg_code_resent = True
                logger(e)
                await conv.send_message(logger("Max attempts achieved, sending new code."))
            except FloodWaitError as e:
                await conv.send_message(logger("%s" % e))
                auth_server.kill()
                raise events.StopPropagation
            except Exception as e:
                await conv.send_message(logger(e))
                auth_server.kill()
                raise events.StopPropagation

        try:
            onedrive.load_session()
            await conv.send_message(logger("Onedrive authorization successful!"))
        except:
            code = await od_code_callback()
            if code:
                try:
                    onedrive.auth(code)
                    await conv.send_message(logger("Onedrive authorization successful!"))
                except Exception as e:
                    await conv.send_message(logger(e))
                    await conv.send_message(logger("Onedrive authorization failed."))
            else:
                await conv.send_message(logger("Onedrive authorization failed."))

    auth_server.kill()

    if not propagate:
        raise events.StopPropagation
