"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from time import time
from io import BytesIO
from telethon import events
from ltorrent_async.client import Client
from ltorrent_async.storage import StorageBase
from ltorrent_async.log import LoggerBase
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name, server_uri
from modules.utils import check_in_group, check_tg_login, check_od_login, cmd_parser
from modules.log import logger

port = int(server_uri.rstrip('/').split(':')[-1])

class MyStorage(StorageBase):
    def __init__(self):
        StorageBase.__init__(self)
        self.file_info_list = []
        self.uploader_session_dict = {}

    async def write(self, file_piece_list, data):
        for file_piece in file_piece_list:
            if file_piece['path'] not in self.uploader_session_dict:
                name = file_piece['path'].split('/')[-1]
                upload_session = onedrive.multipart_upload_session_builder(name)
                for file_info in self.file_info_list:
                    if file_info['path'] == file_piece['path']:
                        self.uploader_session_dict[file_piece['path']] = onedrive.multipart_uploader(upload_session, file_info['length'])
                        break

            file_offset = file_piece["fileOffset"]
            piece_offset = file_piece["pieceOffset"]
            length = file_piece["length"]
            buffer = BytesIO(data[piece_offset : piece_offset + length])
            await onedrive.multipart_upload(
                self.uploader_session_dict[file_piece['path']],
                buffer,
                file_offset
            )

    async def read(self, files, block_offset, block_length):
        pass

class MyLogger(LoggerBase):
    def __init__(self, bot, callback):
        LoggerBase.__init__(self)
        self.bot = bot
        self.callback = callback
        self.message = None
        self.last_call = time()

    async def INFO(self, *args):
        merged_string = ' '.join(map(str, args))
        logger(merged_string)
        if self.message:
            await self.bot.edit_message(self.message, merged_string)
        else:
            self.message = await self.callback(merged_string)

    async def PROGRESS(self, *args):
        now = time()
        if now - self.last_call > 5:
            merged_string = ' '.join(map(str, args))
            logger(merged_string)
            if self.message:
                await self.bot.edit_message(self.message, merged_string)
            else:
                self.message = await self.callback(merged_string)
            self.last_call = now

    async def FILES(self, *args):
        merged_string = ' '.join(map(str, args))
        logger(merged_string)
        await self.callback(merged_string)

    async def DEBUG(self, *args):
        merged_string = ' '.join(map(str, args))
        logger(merged_string)

@tg_bot.on(events.NewMessage(pattern="/magnet", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def magnet_handler(event):
    my_logger = MyLogger(tg_bot, event.respond)
    my_storage = MyStorage()
    client = Client(
        port=port,
        storage=my_storage,
        stdout=my_logger,
        sequential=True
    )

    cmd = cmd_parser(event)
    if len(cmd) == 2:
        # /magnet magnet:?xt=urn:btih:xxxxxxxxxxxx
        if cmd[1].startswith('magnet:?'):
            await client.load(magnet_link=cmd[1])
            # '0' for all
            await client.select_file(selection='0')
            client.storage.file_info_list = client.torrent.file_names
            await client.run()
        else:
            await event.reply('Format wrong.')
    elif len(cmd) == 1:
        await event.reply('Format wrong.')
    elif len(cmd) == 3 and cmd[1] == 'list' and cmd[2].startswith('magnet:?'):
        # /magnet list magnet:?xt=urn:btih:xxxxxxxxxxxx
        await client.load(magnet_link=cmd[2])
        await client.list_file()
    elif len(cmd) >2 and cmd[1].startswith('magnet:?'):
        # /magnet magnet:?xt=urn:btih:xxxxxxxxxxxx 1 3-6 9
        await client.load(magnet_link=cmd[1])
        await client.select_file(selection=' '.join(cmd[2:]))
        client.storage.file_info_list = client.torrent.file_names
        await client.run()

    raise events.StopPropagation