"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from telethon import events
import asyncio
import os
from modules.client import tg_bot, onedrive
from modules.env import tg_user_name
from modules.utils import Callback, Status_Message, check_in_group, check_login, cmd_parser, get_link, get_filename
from modules.log import logger
from modules.transfer import multi_parts_uploader_from_url
from modules.global_var import url_res, analysis_content_not_found, analysis_not_http_or_forbidden, analysis_work_canncelled


@tg_bot.on(events.NewMessage(pattern="/url", incoming=True, from_users=tg_user_name))
@check_in_group
@check_login
async def url_handler(event):
    try:
        cmd = cmd_parser(event)
        url = cmd[1]
        # lest the url is bold
        url = url.strip().strip('*')
    except:
        await event.reply(url_res)
        raise events.StopPropagation

    if not get_link(url):
        await event.reply(logger("Please offer an HTTP url."))
        raise events.StopPropagation

    status_message = await Status_Message.create(event)

    try:
        name, local_response = get_filename(url)
        total_length = int(local_response.headers['Content-Length']) / (1024 * 1024)
    except Exception as e:
        logger(e)
        await event.reply(logger('Error:\n%s'%analysis_content_not_found))
        raise events.StopPropagation

    try:
        logger('upload url: %s' % url)
        progress_url = onedrive.upload_from_url(url, name)
        logger('progress url: %s' % progress_url)
    except Exception as e:
        await event.reply(logger(e))
        raise events.StopPropagation 

    try:
        while True:
            response = onedrive.upload_from_url_progress(progress_url)
            progress = response.content
            if progress['status'] in ['notStarted', 'inProgress', 'completed']:
                percentage = float(progress['percentageComplete'])
                status_message.status = status_message.template % (total_length * percentage / 100, total_length, percentage)
                logger(status_message.status)
                await status_message.update()

                if progress['status'] == 'completed':
                    logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
                    await status_message.finish()
                    break

                await asyncio.sleep(5)
            else:
                raise Exception('status error')

    except Exception as e:
        if 'status' in progress:
            if progress['status'] == 'waiting':
                try:
                    logger('use local uploader to upload from url')
                    callback = Callback(event, status_message)
                    await multi_parts_uploader_from_url(name, local_response, callback)
                    logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
                    await status_message.finish()
                except Exception as sub_e:
                    await status_message.report_error(sub_e, url, progress_url, progress)
            elif progress['status'] == 'failed':
                if 'errorCode' in progress:
                    if progress['errorCode'] == 'ParameterIsTooLong' or progress['errorCode'] == 'NameContainsInvalidCharacters':
                        try:
                            logger('use local uploader to upload from url')
                            callback = Callback(event, status_message)
                            await multi_parts_uploader_from_url(name, local_response, callback)
                            logger("File uploaded to %s"%os.path.join(onedrive.remote_root_path, name))
                            await status_message.finish()
                        except Exception as sub_e:
                            await status_message.report_error(sub_e, url, progress_url, progress)
                    else:
                        if progress['errorCode'] == 'Forbidden':
                            await status_message.report_error(e, url, progress_url, progress, analysis_not_http_or_forbidden)
                        elif progress['errorCode'] == 'NotFound' or progress['errorCode'] == 'operationNotFound':
                            await status_message.report_error(e, url, progress_url, progress, analysis_content_not_found)
                        else:
                            await status_message.report_error(e, url, progress_url, progress)
                else:
                    await status_message.report_error(e, url, progress_url, progress)
            else:
                await status_message.report_error(e, url, progress_url, progress, analysis_work_canncelled)
        else:
            await status_message.report_error(e, url, progress_url, progress, analysis_content_not_found)

    raise events.StopPropagation