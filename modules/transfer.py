"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon.tl import types
import math
import inspect
from io import BytesIO
import asyncio
from modules.client import onedrive
from modules.global_var import PART_SIZE


async def download_part(client, input_location, offset):
    stream = client.iter_download(
        input_location, offset=offset, request_size=PART_SIZE, limit=PART_SIZE
    )
    part = await stream.__anext__()
    await stream.close()
    return part


async def multi_parts_uploader(
    client, document, name, conn_num=5, progress_callback=None
):
    input_location = types.InputDocumentFileLocation(
        id=document.id,
        access_hash=document.access_hash,
        file_reference=document.file_reference,
        thumb_size="",
    )

    upload_session = onedrive.multipart_upload_session_builder(name)
    uploader = onedrive.multipart_uploader(upload_session, document.size)

    task_list = []
    total_part_num = (
        1 if PART_SIZE >= document.size else math.ceil(document.size / PART_SIZE)
    )
    current_part_num = 0
    current_size = 0
    offset = 0
    pre_offset = 0
    if progress_callback:
        cor = progress_callback(current_size, document.size)
        if inspect.isawaitable(cor):
            await cor

    buffer = BytesIO()
    while current_part_num < total_part_num:
        task_list.append(
            asyncio.ensure_future(download_part(client, input_location, offset))
        )
        current_part_num += 1
        if current_part_num < total_part_num:
            offset += PART_SIZE
        if current_part_num % conn_num == 0 or current_part_num == total_part_num:
            for part in await asyncio.gather(*task_list):
                buffer.write(part)
                current_size += len(part)
            task_list.clear()
            buffer.seek(0)
            await onedrive.multipart_upload(uploader, buffer, pre_offset, buffer.getbuffer().nbytes)
            pre_offset = offset
            buffer = BytesIO()
            if progress_callback:
                cor = progress_callback(current_size, document.size)
                if inspect.isawaitable(cor):
                    await cor
    buffer.close()


async def multi_parts_uploader_from_url(name, response, progress_callback=None):
    total_length = int(response.headers['Content-Length'])

    upload_session = onedrive.multipart_upload_session_builder(name)
    uploader = onedrive.multipart_uploader(upload_session, total_length)

    offset = 0
    part_num = 1
    if progress_callback:
        cor = progress_callback(offset, total_length)
        if inspect.isawaitable(cor):
            await cor
    for chunk in response.iter_content(chunk_size=PART_SIZE):
        buffer = BytesIO()
        buffer.write(chunk)
        buffer.seek(0)
        await onedrive.multipart_upload(uploader, buffer, offset, buffer.getbuffer().nbytes)
        offset += buffer.getbuffer().nbytes
        part_num += 1
        if progress_callback and (part_num % 5 ==0 or offset == total_length):
            cor = progress_callback(offset, total_length)
            if inspect.isawaitable(cor):
                await cor