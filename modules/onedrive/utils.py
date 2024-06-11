"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import time
from modules.env import remote_root_path
from modules.global_var import INVALID_COMPONENT, INVALID_NAME


class Dir:
    path = remote_root_path
    last_path = remote_root_path
    is_temp = False

    @classmethod
    def reset(cls):
        cls.path = remote_root_path
        cls.last_path = remote_root_path
        cls.is_temp = False

    @classmethod
    def set_temp_path(cls, path):
        cls.path = path
        cls.is_temp = True

    @classmethod
    def set_perm_path(cls, path):
        cls.path = path
        cls.last_path = path
        cls.is_temp = False

    @classmethod
    def check_temp(cls):
        if cls.is_temp:
            cls.path = cls.last_path
            cls.is_temp = False


def use_id_ext_name(event):
    if "document" in event.media.to_dict():
        return "%d%s" % (event.media.document.id, event.file.ext)
    elif "photo" in event.media.to_dict():
        return "%d%s" % (event.media.photo.id, event.file.ext)


def is_file_name_valid(name):
    if (
        not name
        or any(component in name for component in INVALID_COMPONENT)
        or name in INVALID_NAME
    ):
        return False
    else:
        return True


# https://support.microsoft.com/en-us/office/restrictions-and-limitations-in-onedrive-and-sharepoint-64883a5d-228e-48f5-b3d2-eb39e07630fa
def preprocess_tg_file_name(event):
    name = event.file.name
    if is_file_name_valid(name):
        return name.strip().lstrip("~$")
    else:
        return use_id_ext_name(event)


def preprocess_url_file_name(name):
    if is_file_name_valid(name):
        return name.strip().lstrip("~$")
    else:
        ext = name.split(".")
        if len(ext) > 1:
            ext = ext[-1]
            return f"{int(time.time())}.{ext}"
        else:
            return str(int(time.time()))
