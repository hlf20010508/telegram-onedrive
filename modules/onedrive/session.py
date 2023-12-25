"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import sqlite3
from onedrivesdk.session import Session
from modules.onedrive.database import Database, UserNotFoundException
from modules.global_var import od_session_path


class NoSessionExecption(Exception):
    pass


class SQLiteSession(Session):
    db = Database(path=od_session_path)

    def __init__(
        self,
        username,
        token_type,
        expires_in,
        scope_string,
        access_token,
        client_id,
        auth_server_url,
        redirect_uri,
        refresh_token=None,
        client_secret=None
    ):
        super().__init__(
            token_type=token_type,
            expires_in=expires_in,
            scope_string=scope_string,
            access_token=access_token,
            client_id=client_id,
            auth_server_url=auth_server_url,
            redirect_uri=redirect_uri,
            refresh_token=refresh_token,
            client_secret=client_secret
        )
        self.username = username

    @classmethod
    def new(
        cls,
        username=None,
        token_type=None,
        expires_at=None,
        scope=None,
        access_token=None,
        client_id=None,
        auth_server_url=None,
        redirect_uri=None,
        refresh_token=None,
        client_secret=None
    ):
        session = super().__new__(cls)

        session.username = username
        session.token_type = token_type
        session._expires_at = expires_at
        session.scope = scope
        session.access_token = access_token
        session.client_id = client_id
        session.auth_server_url = auth_server_url
        session.redirect_uri = redirect_uri
        session.refresh_token = refresh_token
        session.client_secret = client_secret

        return session

    def refresh_session(self, expires_in, scope_string, access_token, refresh_token):
        super().refresh_session(
            expires_in=expires_in,
            scope_string=scope_string,
            access_token=access_token,
            refresh_token=refresh_token
        )
        self.db.update_user(
            username=self.username,
            expires_at=self._expires_at,
            scope=self.scope,
            access_token=self.access_token,
            refresh_token=self.refresh_token
        )

    def save_session(self, **save_session_kwargs):
        try:
            self.db.create_table()
        except sqlite3.OperationalError:
            pass

        try:
            self.db.add_user(
                username=self.username,
                token_type=self.token_type,
                expires_at=self._expires_at,
                scope=self.scope,
                access_token=self.access_token,
                client_id=self.client_id,
                auth_server_url=self.auth_server_url,
                redirect_uri=self.redirect_uri,
                refresh_token=self.refresh_token,
                client_secret=self.client_secret
            )

        except sqlite3.IntegrityError:
            self.db.update_user(
                username=self.username,
                token_type=self.token_type,
                expires_at=self._expires_at,
                scope=self.scope,
                access_token=self.access_token,
                client_id=self.client_id,
                auth_server_url=self.auth_server_url,
                redirect_uri=self.redirect_uri,
                refresh_token=self.refresh_token,
                client_secret=self.client_secret
            )

    @classmethod
    def load_session(cls, **save_session_kwargs):
        try:
            record = cls.db.get_current_user()
            return cls.new(
                username=record['username'],
                token_type=record['token_type'],
                expires_at=record['expires_at'],
                scope=record['scope'],
                access_token=record['access_token'],
                client_id=record['client_id'],
                auth_server_url=record['auth_server_url'],
                redirect_uri=record['redirect_uri'],
                refresh_token=record['refresh_token'],
                client_secret=record['client_secret']
            )
        except (UserNotFoundException, sqlite3.OperationalError):
            raise NoSessionExecption("No session found.")

    def logout(self):
        self.db.delete_user(self.username)
        try:
            record = self.db.get_an_user()

            self.username = record['username']
            self.token_type = record['token_type']
            self._expires_at = record['expires_at']
            self.scope = record['scope']
            self.access_token = record['access_token']
            self.client_id = record['client_id']
            self.auth_server_url = record['auth_server_url']
            self.redirect_uri = record['redirect_uri']
            self.refresh_token = record['refresh_token']
            self.client_secret = record['client_secret']

            self.db.set_current_user(self.username)
            return True

        except UserNotFoundException:
            self.db.clear_current_user()
            return False
    
    def change_user(self, username):
        record = self.db.get_user(username)

        self.username = record['username']
        self.token_type = record['token_type']
        self._expires_at = record['expires_at']
        self.scope = record['scope']
        self.access_token = record['access_token']
        self.client_id = record['client_id']
        self.auth_server_url = record['auth_server_url']
        self.redirect_uri = record['redirect_uri']
        self.refresh_token = record['refresh_token']
        self.client_secret = record['client_secret']

        self.db.set_current_user(self.username)
        return self.username

    @property
    def users(self):
        return self.db.show_all_users()
    
    @property
    def current_user(self):
        return self.db.get_current_user()['username']
