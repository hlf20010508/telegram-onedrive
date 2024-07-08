"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from modules.env import reverse_proxy

TG_LOGIN_MAX_ATTEMPTS = 3

PART_SIZE = 2 * 1024 * 1024

FILE_PARAM_NAME_LIST = ["name", "filename", "file_name", "title", "file"]

LOGS_LINES_PER_PAGE = 50

TG_BOT_SESSION_PATH = "session/bot.session"
TG_CLIENT_SESSION_PATH = "session/user.session"
OD_SESSION_PATH = "session/onedrive.session"

BASE_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
}


INVALID_COMPONENT = ["#", '"', "*", ":", "<", ">", "?", "/", "\\", "|", "_vti_"]
INVALID_NAME = [
    ".lock",
    "CON",
    "PRN",
    "AUX",
    "NUL",
    "COM0",
    "COM1",
    "COM2",
    "COM3",
    "COM4",
    "COM5",
    "COM6",
    "COM7",
    "COM8",
    "COM9",
    "LPT0",
    "LPT1",
    "LPT2",
    "LPT3",
    "LPT4",
    "LPT5",
    "LPT6",
    "LPT7",
    "LPT8",
    "LPT9",
    "desktop.ini",
]

protocol = "http" if reverse_proxy else "https"

TG_CODE_URL = f"{protocol}://127.0.0.1:8080/tg"
OD_CODE_URL = f"{protocol}://127.0.0.1:8080/auth"
