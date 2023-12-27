"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
from modules.client import tg_bot
from modules.env import tg_user_name
from modules.utils import check_in_group, check_tg_login, check_od_login, cmd_parser, CMDException
from modules.global_var import dir_res
from modules.onedrive.dir import Dir


@tg_bot.on(events.NewMessage(pattern="/dir", incoming=True, from_users=tg_user_name))
@check_in_group
@check_tg_login
@check_od_login
async def dir_handler(event):
    cmd = cmd_parser(event)
    # /dir
    try:
        if len(cmd) == 1:
            if not Dir.is_temp:
                await event.respond(f'Current directory is `{Dir.path}`')
            else:
                await event.respond(f'Current temporary directory is `{Dir.path}`')
        elif len(cmd) == 2:
            sub_cmd = cmd[1]
            # /dir reset
            if sub_cmd == 'reset':
                Dir.reset()
                await event.respond(f'Directory reset to default `{Dir.path}`')
            # /dir $remote_path
            else:
                remote_path = cmd[1].strip().strip('*')
                if remote_path.startswith('/'):
                    Dir.set_perm_path(remote_path)
                    await event.respond(f'Directory set to `{Dir.path}`')
                else:
                    await event.respond('Directory path should start with /')
        
        elif len(cmd) == 3:
            sub_cmd = cmd[1]
            if sub_cmd == 'temp':
                sub_cmd = cmd[2]
                # /dir temp $remote_path
                if sub_cmd != 'cancel':
                    remote_path = cmd[2].strip().strip('*')
                    if remote_path.startswith('/'):
                        Dir.set_temp_path(remote_path)
                        await event.respond(f'Temporary directory set to `{Dir.path}`')
                    else:
                        await event.respond('Directory path should start with /')
                # /dir temp cancel
                else:
                    Dir.check_temp()
                    await event.respond(f'Directory restored to `{Dir.path}')
            else:
                raise CMDException('Sub command of /dir temp wrong.')
        else:
            raise CMDException('Command /dir format wrong.')
    except CMDException:
        await event.reply(dir_res)

    raise events.StopPropagation
