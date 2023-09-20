"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

PART_SIZE = 2 * 1024 * 1024


cmd_helper = '''
- /auth: Authorize for Telegram and OneDrive.
- /clear: Clear all history except status message.

- `/links` message_link range: Transfer sequential restricted content.
- `/url` file_url: Upload file through url.
- `/autoDelete true` to auto delete message.
- `/autoDelete false` to not auto delete message.
'''


start_res = '''
Transfer files to Onedrive.

Forward or upload files to me, or pass message link to transfer restricted content from group or channel.
%s
- /help: Ask for help.
'''%cmd_helper


help_res = '''
%s
- To transfer files, forward or upload to me.
- To transfer restricted content, right click the content, copy the message link, and send to me.
- Uploading through url will call Onedrive's API, which means Onedrive's server will visit the url and download the file for you. If the url is invalid to OneDrive, the bot will try using bot's uploader to transfer.
'''%cmd_helper


check_in_group_res = '''
This bot must be used in a Group or Channel!

Add this bot to a Group or Channel as Admin, and give it ability to Delete Messages.
'''


not_login_res = '''
You haven't logined to Telegram.
'''


auto_delete_res = '''
Command `/autoDelete` Usage:

`/autoDelete true` to auto delete message.
`/autoDelete false` to not auto delete message.
'''


links_res = '''
Command `/links` format wrong.

Usage: `/links` message_link range
'''


url_res = '''
Command `/url` format wrong.

Usage: `/url` file_url
'''
