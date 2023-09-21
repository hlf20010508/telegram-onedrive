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
from modules.env import tg_user_name, tg_user_phone, server_uri
from modules.utils import check_in_group
from modules.client import tg_bot, tg_client, onedrive


class Code_Callback:
    def __init__(self, conv, type):
        self.conv = conv
        self.type = type

    @property
    async def tg_code(self):
        await self.conv.send_message(
            "Please visit %s to input your code to login to Telegram." % server_uri
        )
        while True:
            res = requests.get(
                url=os.path.join(server_uri, "tg"),
                verify=False
            ).json()
            if res["success"]:
                break
            await asyncio.sleep(1)
        return res["code"]
    
    @property
    async def od_code(self):
        auth_url = onedrive.get_auth_url()
        await self.conv.send_message(
            "Here are the authorization url of OneDrive:\n\n%s" % auth_url
        )
        while True:
            res = requests.get(
                url=os.path.join(server_uri, "auth"),
                params={"get": True},
                verify=False,
            ).json()
            if res["success"]:
                break
            await asyncio.sleep(1)
        return res["code"]
    
    @property
    async def code(self):
        if self.type == "tg":
            return await self.tg_code
        elif self.type == "od":
            return await self.od_code

    async def __call__(self):
        return await self.code


@tg_bot.on(events.NewMessage(pattern="/auth", incoming=True, from_users=tg_user_name))
@check_in_group
async def auth_handler(event, propagate=False):
    auth_server = subprocess.Popen(('python', 'server/auth_server.py'))

    async with tg_bot.conversation(event.chat_id) as conv:
        tg_code_callback = Code_Callback(conv, 'tg')
        od_code_callback = Code_Callback(conv, 'od')

        await conv.send_message("Logining into Telegram...")
        global tg_client
        tg_client = await tg_client.start(tg_user_phone, code_callback=tg_code_callback)
        await conv.send_message("Login to Telegram successful!")

        try:
            onedrive.load_session()
        except:
            code = await od_code_callback()
            onedrive.auth(code)
        await conv.send_message("Onedrive authorization successful!")

    auth_server.kill()

    if not propagate:
        raise events.StopPropagation
