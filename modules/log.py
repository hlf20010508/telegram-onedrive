"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from datetime import datetime
from traceback import print_exc
from io import StringIO

log_path = 'log'

def logger(message):
    with open(log_path, 'a') as log_file:
        time = datetime.now()
        if isinstance(message, Exception):
            message = StringIO()
            print_exc(file=message)
            message = message.getvalue()
        template = '%s\n%s\n'
        print(template % (time, message), end='')
        log_file.write(template % (time, message))
    return message