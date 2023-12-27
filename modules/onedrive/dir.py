from modules.env import remote_root_path


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