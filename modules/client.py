"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import os
from telethon import TelegramClient
from modules.onedrive import Onedrive
from modules.env import tg_api_id, tg_api_hash, tg_bot_token, od_client_id, od_client_secret, server_uri
from modules.global_var import TG_BOT_SESSION_PATH, TG_CLIENT_SESSION_PATH

def init_tg_client():
    return TelegramClient(TG_CLIENT_SESSION_PATH, tg_api_id, tg_api_hash, sequential_updates=True)

tg_bot = TelegramClient(TG_BOT_SESSION_PATH, tg_api_id, tg_api_hash, sequential_updates=True).start(
    bot_token=tg_bot_token
)

tg_client = init_tg_client()

onedrive = Onedrive(
    client_id=od_client_id,
    client_secret=od_client_secret,
    redirect_uri=os.path.join(server_uri, "auth")
)