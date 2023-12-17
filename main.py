"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import os
import urllib3


def main():
    urllib3.disable_warnings()
    if not os.path.exists('session'):
        os.mkdir('session')

    from modules.client import tg_bot
    
    from modules.handlers.start import start_handler
    from modules.handlers.help import help_handler
    from modules.handlers.auth import auth_handler
    from modules.handlers.auto_delete import auto_delete_handler
    from modules.handlers.clear import clear_handler
    from modules.handlers.logs import logs_handler
    from modules.handlers.links import links_handler
    from modules.handlers.url import url_handler
    from modules.handlers.drive import drive_handler
    from modules.handlers.dir import dir_handler
    from modules.handlers.magnet import magnet_handler
    from modules.handlers.transfer import transfer_handler

    tg_bot.run_until_disconnected()


if __name__ == "__main__":
    main()
