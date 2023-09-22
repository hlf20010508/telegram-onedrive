"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import os
from telethon import TelegramClient
from modules.onedrive import Onedrive
from modules.env import tg_api_id, tg_api_hash, tg_bot_token, od_client_id, od_client_secret, server_uri, remote_root_path
from modules.global_var import tg_bot_session_path, tg_client_session_path

tg_bot = TelegramClient(tg_bot_session_path, tg_api_id, tg_api_hash, sequential_updates=True).start(
    bot_token=tg_bot_token
)

tg_client = TelegramClient(tg_client_session_path, tg_api_id, tg_api_hash, sequential_updates=True)

onedrive = Onedrive(
    client_id=od_client_id,
    client_secret=od_client_secret,
    redirect_uri=os.path.join(server_uri, "auth"),
    remote_root_path=remote_root_path,
)