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
        message = '%s\n%s\n' % (time, message)
        print(message, end='')
        log_file.write(message)
    return message