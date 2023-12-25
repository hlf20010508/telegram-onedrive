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
    from modules.handlers.clear_logs import clear_logs_handler
    from modules.handlers.clear import clear_handler
    from modules.handlers.logs import logs_handler
    from modules.handlers.logout import logout_handler
    from modules.handlers.links import links_handler
    from modules.handlers.url import url_handler
    from modules.handlers.add_user import add_user_handler
    from modules.handlers.list_user import list_user_handler
    from modules.handlers.change_user import change_user_handler
    from modules.handlers.transfer import transfer_handler

    tg_bot.run_until_disconnected()


if __name__ == "__main__":
    main()
