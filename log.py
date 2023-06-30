from datetime import datetime

log_path = 'log'

def logger(message):
    with open(log_path, 'a') as log_file:
        time = datetime.now()
        message = '%s\n%s\n'%(time, message)
        print(message)
        log_file.write(message)