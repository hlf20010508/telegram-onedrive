"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

import sqlite3
import os

STRUCTURE_VERSION = "1"


class UserNotFoundException(Exception):
    pass


class Database:
    def __init__(self, path):
        self.path = path
        self.connection = None
        self.cursor = None

        self.init()
        if not self.is_valid():
            self.close()
            os.remove(self.path)
            self.init()

    def init(self):
        self.connection = sqlite3.connect(self.path)
        self.cursor = self.connection.cursor()

    def is_valid(self):
        try:
            command = """
                select version from structure_version limit 1;
            """
            self.execute(command)
            result = self.cursor.fetchone()
            if result:
                version = result[0]
                if version == STRUCTURE_VERSION:
                    return True
                else:
                    return False
            else:
                return False
        except:
            return False

    def create_table(self):
        command = """
            create table session(
                id integer primary key autoincrement,
                username char(255) unique not null,
                token_type char(255) not null,
                expires_at float not null,
                scope char(255) not null,
                access_token char(255) not null,
                client_id char(255) not null,
                auth_server_url char(255) not null,
                redirect_uri char(255) not null,
                refresh_token char(255),
                client_secret char(255)
            );
        """
        self.execute(command)

        command = """
            create table current_user(
                user_id integer primary key,
                foreign key(user_id) references session(id)
            );
        """
        self.execute(command)

        command = """
            create table structure_version(
                version char(10) primary key
            );
        """
        self.execute(command)

        command = """
            insert into structure_version(version) values(?);
        """
        self.execute(command, params=(STRUCTURE_VERSION,))

        command = """
            create trigger prevent_username_update
            before update of username on session
            for each row
            begin
                select raise(fail, 'Cannot update the username field.');
            end;
        """
        self.execute(command)

        self.commit()

    def set_current_user(self, username):
        user_id = self.get_user_id(username)
        command = """
            select count(*) from current_user;
        """
        self.execute(command)
        if self.cursor.fetchone()[0] > 0:
            command = """
                update current_user set user_id=? where user_id=(
                    select user_id from current_user limit 1
                );
            """

            self.execute(command, params=(user_id,))
        else:
            command = """
                insert into current_user(user_id) values(?);
            """
            self.execute(command, params=(user_id,))
        self.commit()

    def get_user_id(self, username):
        command = """
            select id from session where username=?;
        """

        self.execute(command, params=(username,))
        result = self.cursor.fetchone()
        if result:
            return result[0]
        else:
            raise UserNotFoundException("Cannot find user through username.")

    def get_current_user(self):
        command = """
            select user_id from current_user limit 1;
        """
        self.execute(command)
        result = self.cursor.fetchone()

        if result:
            user_id = result[0]
            command = """
                select

                username,
                token_type,
                expires_at,
                scope,
                access_token,
                client_id,
                auth_server_url,
                redirect_uri,
                refresh_token,
                client_secret

                from session where id=?;
            """

            self.execute(command, params=(user_id,))
            record = self.cursor.fetchone()
            return {
                "username": record[0],
                "token_type": record[1],
                "expires_at": record[2],
                "scope": record[3].split(" "),
                "access_token": record[4],
                "client_id": record[5],
                "auth_server_url": record[6],
                "redirect_uri": record[7],
                "refresh_token": record[8],
                "client_secret": record[9],
            }
        else:
            raise UserNotFoundException("No current user record.")

    def get_an_user(self):
        command = """
            select username from session limit 1;
        """
        self.execute(command)
        result = self.cursor.fetchone()

        if result:
            username = result[0]
            return self.get_user(username)
        else:
            raise UserNotFoundException("No user record.")

    def add_user(
        self,
        username,
        token_type,
        expires_at,
        scope,
        access_token,
        client_id,
        auth_server_url,
        redirect_uri,
        refresh_token=None,
        client_secret=None,
    ):
        command = """
            insert into session(
                username,
                token_type,
                expires_at,
                scope,
                access_token,
                client_id,
                auth_server_url,
                redirect_uri,
                refresh_token,
                client_secret
            )
            values(?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        """

        scope = " ".join(scope)
        self.execute(
            command,
            params=(
                username,
                token_type,
                expires_at,
                scope,
                access_token,
                client_id,
                auth_server_url,
                redirect_uri,
                refresh_token,
                client_secret,
            ),
        )

        command = """
            select count(*) from current_user;
        """
        self.execute(command)
        if self.cursor.fetchone()[0] == 0:
            self.set_current_user(username)
        self.commit()

    def get_user(self, username):
        command = """
            select

            username,
            token_type,
            expires_at,
            scope,
            access_token,
            client_id,
            auth_server_url,
            redirect_uri,
            refresh_token,
            client_secret

            from session where username=?;
        """

        self.execute(command, params=(username,))
        result = self.cursor.fetchone()

        if result:
            return {
                "username": result[0],
                "token_type": result[1],
                "expires_at": result[2],
                "scope": result[3].split(" "),
                "access_token": result[4],
                "client_id": result[5],
                "auth_server_url": result[6],
                "redirect_uri": result[7],
                "refresh_token": result[8],
                "client_secret": result[9],
            }
        else:
            raise UserNotFoundException("Cannot find user through username.")

    def update_user(self, username, **kwargs):
        if "scope" in kwargs:
            kwargs["scope"] = " ".join(kwargs["scope"])

        param_keywords = ",".join([keyword + "=?" for keyword in kwargs])
        command = (
            """
            update session set
            %s
            where username=?;
        """
            % param_keywords
        )

        self.execute(command, params=(*kwargs.values(), username))
        self.commit()

    def delete_user(self, username):
        command = """
            delete from session where username=?;
        """

        self.execute(command, params=(username,))
        self.commit()

    def clear_current_user(self):
        command = """
            delete from current_user;
        """

        self.execute(command)
        self.commit()

    def show_all_users(self):
        command = """
            select username from session;
        """
        self.execute(command)
        result = self.cursor.fetchall()
        if result:
            return [record[0] for record in result]
        else:
            raise UserNotFoundException("No user record.")

    def execute(self, command, params=()):
        self.cursor.execute(command, params)

    def commit(self):
        self.connection.commit()

    def close(self):
        self.connection.close()
