"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import os
import asyncio
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login, cmd_parser
from modules.log import log_path
from modules.global_var import LOGS_LINES_PER_PAGE
from modules.res import logs_res


class Tail_File_Page:
    def __init__(self, path, lines_per_page):
        self.pos = 0
        self.lines_per_page = lines_per_page
        self.file = open(path, "rb")

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.file.close()

    def read_all(self):
        while True:
            string = ""
            for _ in range(self.lines_per_page):
                line = self.file.readline().decode()
                if line == "":
                    break
                string += line
            if string == "":
                break
            yield string

    def read_pages(self, pages):
        self._seek_lines(pages * self.lines_per_page)
        while pages:
            pages -= 1
            string = self._read_lines(self.lines_per_page)
            if string == "":
                break
            yield string

    def _read_lines(self, lines_num):
        string = ""
        while lines_num:
            lines_num -= 1
            line = self.file.readline().decode()
            if line == "":
                break
            string += line
        return string

    def _seek_lines(self, lines_num):
        while lines_num:
            try:
                self.pos -= 1
                self.file.seek(self.pos, os.SEEK_END)
                if self.file.read(1) == b"\n":
                    lines_num -= 1
            except:
                self.file.seek(0, os.SEEK_SET)
                break


@tg_bot.on(events.NewMessage(pattern="/logs", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def logs_handler(event):
    if not os.path.exists(log_path):
        await event.respond("Logs not found.")
        raise events.StopPropagation
    cmd = cmd_parser(event)

    # /logs
    if len(cmd) == 1:
        with Tail_File_Page(log_path, LOGS_LINES_PER_PAGE) as file:
            await event.respond("Outputting logs...")
            for logs in file.read_all():
                await event.respond(logs)
                await asyncio.sleep(1)
        await event.respond("Finished.")

    elif len(cmd) == 2:
        sub_cmd = cmd[1]
        # /logs clear
        if sub_cmd == "clear":
            if os.path.exists(log_path):
                os.system("rm %s" % log_path)
                await event.respond("Logs cleared.")
            else:
                await event.respond("Logs not found.")

        # /logs $range
        else:
            try:
                pages = int(sub_cmd)
            except ValueError:
                await event.reply("Logs page range should be integer.")
                raise events.StopPropagation

            with Tail_File_Page(log_path, LOGS_LINES_PER_PAGE) as file:
                await event.respond("Outputting logs...")
                for logs in file.read_pages(pages):
                    await event.respond(logs)
                    await asyncio.sleep(1)
            await event.respond("Finished.")
    else:
        await event.respond(logs_res)
    raise events.StopPropagation
