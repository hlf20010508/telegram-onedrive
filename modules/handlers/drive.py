"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import subprocess
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, cmd_parser
from modules.handlers.auth import od_auth
from modules.res import drive_res


@tg_bot.on(events.NewMessage(pattern="/drive", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
async def drive_handler(event):
    cmd = cmd_parser(event)
    # /drive
    if len(cmd) == 1:
        message_str = ""

        try:
            users = onedrive.session.users
            current_user = onedrive.session.current_user
        except:
            await event.respond("No account found.")
            raise events.StopPropagation

        for i in range(len(users)):
            message_str += f"{i + 1}. {users[i]}\n"
        message_str += f"\nCurrent account is {current_user}\n"
        message_str += f"\n```/drive $index```\n To change current account."

        await event.respond(message_str)

    elif len(cmd) == 2:
        sub_cmd = cmd[1]
        # /drive add
        if sub_cmd == "add":
            auth_server = subprocess.Popen(("python", "server/auth_server.py"))
            async with tg_bot.conversation(event.chat_id) as conv:
                await od_auth(conv)
            auth_server.kill()

        # /drive logout
        elif sub_cmd == "logout":
            has_other_user = onedrive.logout()
            if has_other_user:
                current_user = onedrive.session.current_user
                logout_res = (
                    f"OneDrive logout successfully.\nCurrent account is {current_user}"
                )
            else:
                logout_res = "OneDrive logout successfully."
            await event.respond(logout_res)

        # /drive $index
        else:
            try:
                index = int(sub_cmd) - 1
            except ValueError:
                await event.reply("Account index should be integer.")
                raise events.StopPropagation

            try:
                users = onedrive.session.users
                old_user = onedrive.session.current_user
            except:
                await event.respond("No account found.")
                raise events.StopPropagation

            if index < len(users):
                new_user = onedrive.session.change_user(users[index])
                if old_user != new_user:
                    await event.respond(
                        f"Changed account from\n{old_user}\nto\n{new_user}"
                    )
                else:
                    await event.respond("Same account, nothing to change.")
            else:
                await event.reply("Account index out of range.")

    # /drive logout $index
    elif len(cmd) == 3:
        sub_cmd = cmd[1]
        if sub_cmd == "logout":
            try:
                index = int(cmd[2]) - 1
            except ValueError:
                await event.reply("Account index should be integer.")
                raise events.StopPropagation

            try:
                users = onedrive.session.users
            except:
                await event.respond("No account found.")
                raise events.StopPropagation

            if index < len(users):
                should_show_current_user = (
                    True if users[index] == onedrive.session.username else False
                )
                onedrive.logout(users[index])
                if should_show_current_user:
                    current_user = onedrive.session.current_user
                    logout_res = f"OneDrive logout successfully.\nCurrent account is {current_user}"
                else:
                    logout_res = "OneDrive logout successfully."
                await event.respond(logout_res)
            else:
                await event.reply("Account index out of range.")
        else:
            await event.reply(drive_res)
    raise events.StopPropagation
