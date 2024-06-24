"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

start_res = """
Transfer files to Onedrive.

Forward or upload files to me, or pass message link to transfer restricted content from group or channel.

- /help: Ask for help.
"""


help_res = """
- /auth to authorize for Telegram and OneDrive.
- /clear to clear all history except status message.
- /autoDelete to toggle whether bot should auto delete message.
- /logs to send log file.
- /drive to list all OneDrive accounts.
- /dir to show current OneDrive directory.

```/links $message_link $range```
To transfer sequential restricted content.
```/url $file_url```
To upload file through url.
```/logs clear```
To clear logs.
```/drive add```
To add a OneDrive account.
```/drive $index```
To change the OneDrive account.
```/drive logout```
To logout current OneDrive account.
```/drive logout $index```
To logout specified OneDrive account.
```/dir $remote_path```
To set OneDrive directory.
```/dir temp $remote_path```
To set temporary OneDrive directory.
```/dir temp cancel```
To restore OneDrive directory to the previous one.
```/dir reset```
To reset OneDrive directory to default.

- To transfer files, forward or upload to me.
- To transfer restricted content, right click the content, copy the message link, and send to me.
- Tap Status on replied status message to locate current job.
- To upload files through url, the headers of the file response must includes `Content-Length`.
- Each log page contains 50 lines of logs.
- Support files with extension .t2o as scripts.

See [example](https://github.com/hlf20010508/telegram-onedrive#example).
"""


check_in_group_res = """
This bot must be used in a Group!

Add this bot to a Group as Admin, and give it ability to Delete Messages.
"""


tg_not_login_res = """
You haven't logined to Telegram.
"""


od_not_login_res = """
You haven't logined to OneDrive.
"""


links_res = """
Command /links format wrong.

Usage:
```/links $message_link $range```
"""


url_res = """
Command /url format wrong.

Usage:
```/url $file_url```
"""


logs_res = """
Command /logs format wrong.

Usage:
```/logs```

```/logs clear```
"""


drive_res = """
Command /drive format wrong.

Usage:
```/drive```

```/drive add```

```/drive $index```

```/drive logout```

```/drive logout $index```
"""


dir_res = """
Command /dir format wrong.

Usage:
```/dir```

```/dir reset```

```/dir $remote_path```

```/dir temp $remote_path```

```/dir temp cancel```
"""
