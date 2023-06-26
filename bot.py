import os
from time import time
from telegram import Update
from telegram.ext import (
    filters,
    ApplicationBuilder,
    ContextTypes,
    CommandHandler,
    MessageHandler,
    ConversationHandler,
)
from log import logger
from onedrive import Onedrive


# telegram
TOKEN = os.environ['TOKEN']

# onedrive
client_id = os.environ['client_id']
client_secret = os.environ['client_secret']
redirect_uri = os.environ['redirect_uri']
remote_root_path = os.environ.get('remote_root_path', '/')

onedrive = Onedrive(
    client_id=client_id,
    client_secret=client_secret,
    redirect_uri=redirect_uri,
    remote_root_path=remote_root_path,
)

help_text = """
`/auth` to authorize for OneDrive
"""

start_text = """
Upload files to Onedrive.
`/auth` to authorize for OneDrive
`/help` for help.
"""

AUTH_CODE = 0

temp_dir = 'temp'

if not os.path.exists(temp_dir):
    os.mkdir(temp_dir)

async def start(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        logger("User %s started a conversation.\n" % chat_id)
        await context.bot.send_message(chat_id=chat_id, text=start_text)
    except Exception as e:
        logger("In start:\n%\n" % e)


async def help(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        logger("User %s requested help.\n" % chat_id)
        await context.bot.send_message(chat_id=chat_id, text=help_text)
    except Exception as e:
        logger("In help:\n%\n" % e)


async def auth_url(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        logger("User %s requested auth.\n" % chat_id)
        auth_url = onedrive.get_auth_url()
        await update.message.reply_text(
            "Here are the authorization url:\n\n%s\n\nPlease enter the returned code."%auth_url
        )
        return AUTH_CODE
    except Exception as e:
        logger("In auth:\n%\n" % e)


async def auth_code(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        code = update.message.text
        logger("User %s requested auth_code.\n" % chat_id)
        onedrive.auth(code)
        await update.message.reply_text("Authorization successful!")
        logger("Authorization successful!\n")
        return ConversationHandler.END
    except Exception as e:
        logger("In auth_code:\n%\n" % e)


async def auth_cancel(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        logger("User %s requested auth_cancel.\n" % chat_id)
        await update.message.reply_text("Authorization canceled.")
        logger("Authorization canceled.\n")
        return ConversationHandler.END
    except Exception as e:
        logger("In auth_cancel:\n%\n" % e)


async def video(update: Update, context: ContextTypes().DEFAULT_TYPE):
    try:
        chat_id = update.effective_chat.id
        message_id = update.message.message_id
        content = update.message.video
        logger("User %s sent a video")
        file = await content.get_file()
        ext = file.file_path.split(".")[-1]
        name = "%d.%s" % (int(time()), ext)
        file_path = os.path.join(temp_dir, name)
        await file.download_to_drive(file_path)
        logger("Video was downloaded to %s" % file_path)
        remote_path = onedrive.upload(file_path)
        logger("Video was uploaded to %s" % remote_path)
        os.remove(file_path)
        logger("Video was removed")
        await context.bot.delete_message(chat_id=chat_id, message_id=message_id)
        logger("Message was removed")
    except Exception as e:
        logger("In video:\n%\n" % e)


if __name__ == "__main__":
    try:
        application = ApplicationBuilder().token(TOKEN).build()

        start_handler = CommandHandler("start", start)
        help_handler = CommandHandler("help", help)
        video_handler = MessageHandler(filters.VIDEO, video)
        conv_handler = ConversationHandler(
            entry_points=[CommandHandler("auth", auth_url)],
            states={0: [MessageHandler(filters.TEXT, auth_code)]},
            fallbacks=[CommandHandler("auth_cancel", auth_cancel)],
        )

        application.add_handler(start_handler)
        application.add_handler(help_handler)
        application.add_handler(conv_handler)
        application.add_handler(video_handler)

        application.run_polling(allowed_updates=Update.ALL_TYPES)
    except Exception as e:
        logger("In main:\n%\n" % e)
