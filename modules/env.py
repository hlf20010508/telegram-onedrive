"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import os

# auth server
server_uri = os.environ["server_uri"]
# telegram api
tg_api_id = int(os.environ["tg_api_id"])
tg_api_hash = os.environ["tg_api_hash"]
tg_user_phone = os.environ["tg_user_phone"]
tg_user_password = os.environ.get("tg_user_password", None)
tg_user_name = os.environ.get("tg_user_name", None)
if tg_user_name:
    tg_user_name = tg_user_name.split(",")
# telegram bot
tg_bot_token = os.environ["tg_bot_token"]
# onedrive
od_client_id = os.environ["od_client_id"]
od_client_secret = os.environ["od_client_secret"]
remote_root_path = os.environ.get("remote_root_path", "/")

delete_flag = True if os.environ.get("delete_flag", "false") == "true" else False
