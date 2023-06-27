from telethon import TelegramClient, events
import os
from onedrive import Onedrive
from time import sleep
import requests
import urllib3

urllib3.disable_warnings()

# telegram api
tg_api_id = int(os.environ['tg_api_id'])
tg_api_hash = os.environ['tg_api_hash']
tg_user_phone = os.environ['tg_user_phone']
tg_login_uri = os.environ['tg_login_uri']

# telegram bot
tg_bot_token = os.environ['tg_bot_token']

tg_bot = TelegramClient('bot', tg_api_id, tg_api_hash, sequential_updates=True).start(bot_token=tg_bot_token)
tg_client = TelegramClient('user', tg_api_id, tg_api_hash, sequential_updates=True)

# onedrive
od_client_id = os.environ['client_id']
od_client_secret = os.environ['client_secret']
redirect_uri = os.environ['redirect_uri']
remote_root_path = os.environ.get('remote_root_path', '/')

onedrive = Onedrive(
    client_id=od_client_id,
    client_secret=od_client_secret,
    redirect_uri=redirect_uri,
    remote_root_path=remote_root_path,
)

temp_dir = 'temp'

if not os.path.exists(temp_dir):
    os.mkdir(temp_dir)

@tg_bot.on(events.NewMessage(pattern='/start'))
async def start(event):
    """Send a message when the command /start is issued."""
    await event.respond("Upload files to Onedrive.\n`/auth` to authorize for OneDrive.\n`/help` for help.")
    raise events.StopPropagation

@tg_bot.on(events.NewMessage(pattern='/help'))
async def help(event):
    """Send a message when the command /start is issued."""
    await event.respond("`/auth` to authorize for OneDrive.")
    raise events.StopPropagation

@tg_bot.on(events.NewMessage(pattern='/auth'))
async def auth(event):
    async with tg_bot.conversation(event.chat_id) as conv:

        async def tg_code_callback():
            await conv.send_message("Please visit %s to input your code."%tg_login_uri)
            res = requests.get(url=os.path.join(tg_login_uri, 'tg'), verify=False).json()
            while not res['success']:
                sleep(1)
                res = requests.get(url=os.path.join(tg_login_uri, 'tg'), verify=False).json()
            return res['code']
        
        def od_code_callback():
            res = requests.get(url=os.path.join(tg_login_uri, 'auth'), params={'get': True}, verify=False).json()
            while not res['success']:
                sleep(1)
                res = requests.get(url=os.path.join(tg_login_uri, 'auth'), params={'get': True}, verify=False).json()
                print(res)
            return res['code']
        
        global tg_client
        tg_client = await tg_client.start(tg_user_phone, code_callback=tg_code_callback)
        auth_url = onedrive.get_auth_url()
        await conv.send_message("Here are the authorization url:\n\n%s"%auth_url)
        code = od_code_callback()
        onedrive.auth(code)
        await conv.send_message("Authorization successful!")
    raise events.StopPropagation

@tg_bot.on(events.NewMessage)
async def transfer(event):
    async def upload(name, message):
        local_path = await message.download_media(file=os.path.join(temp_dir, name))
        print('File saved to', local_path)
        remote_path = onedrive.upload(local_path)
        print('File uploaded to', remote_path)
        os.remove(local_path)
        await message.delete()
    if event.media:
        onedrive_bot = await tg_bot.get_me()
        onedrive_bot = await tg_client.get_entity('@%s'%onedrive_bot.username)
        async for message in tg_client.iter_messages(onedrive_bot):
            try:
                if message.media.document:
                    if event.media.document.id == message.media.document.id:
                        name = "%d%s" % (event.media.document.id, event.file.ext)
                        await upload(name, message)
                        break
            except:
                pass

            try:
                if message.media.photo:
                    if event.media.photo.id == message.media.photo.id:
                        name = "%d%s" % (event.media.photo.id, event.file.ext)
                        await upload(name, message)
                        break
            except:
                pass

def main():
    tg_bot.run_until_disconnected()

if __name__ == '__main__':
    main()
