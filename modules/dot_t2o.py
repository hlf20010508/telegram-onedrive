"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from modules.client import tg_client
from modules.utils import delete_message

KEYS = ["link", "url"]


async def parse_t2o_to_dict(content):
    result = {}
    for key in KEYS:
        result[key] = []
    current_key = ""

    for line in content.splitlines():
        line.strip()

        if (key := line.lstrip("[").rstrip("]")) in KEYS:
            current_key = key
        elif line and current_key == "link":
            line = line.split()
            if len(line) == 1:
                result[current_key].append({"content": line[0], "range": 1})
            elif len(line) == 2:
                result[current_key].append({"content": line[0], "range": int(line[1])})
        elif line and current_key == "url":
            result[current_key].append(line)

    return result


async def parse_t2o(event, message):
    content = (await message.download_media(file=bytes)).decode()
    t2o_dict = await parse_t2o_to_dict(content)
    print(t2o_dict)

    for key in KEYS:
        if key == "link":
            for link in t2o_dict[key]:
                if link["range"] == 1:
                    await tg_client.send_message(
                        event.chat_id,
                        message=link["content"],
                    )
                elif link["range"] > 1:
                    await tg_client.send_message(
                        event.chat_id,
                        message="/links %s %d" % (link["content"], link["range"]),
                    )
        elif key == "url":
            for url in t2o_dict[key]:
                await tg_client.send_message(
                    event.chat_id,
                    message=f"/url {url}",
                )

    await delete_message(event)
