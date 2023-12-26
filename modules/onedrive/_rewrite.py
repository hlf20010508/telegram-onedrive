"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from onedrivesdk import AuthProvider
from onedrivesdk.request.item_create_session import ItemCreateSessionRequestBuilder
from onedrivesdk.request.item_request_builder import ItemRequestBuilder
from onedrivesdk.http_response import HttpResponse
import json


def authenticate(self, code, redirect_uri, client_secret, resource=None):
        params = {
            "client_id": self.client_id,
            "redirect_uri": redirect_uri,
            "client_secret": client_secret,
            "code": code,
            "response_type": "code",
            "grant_type": "authorization_code"
        }

        if resource is not None:
            params["resource"] = resource

        auth_url = self._auth_token_url
        headers = {"Content-Type": "application/x-www-form-urlencoded"}
        response = self._http_provider.send(
            method="POST",
            headers=headers,
            url=auth_url,
            data=params
        )

        rcont = json.loads(response.content)

        headers = {'Authorization' : f"Bearer {rcont['access_token']}"}
        response = self._http_provider.send(
            method='GET',
            headers=headers,
            url=self._http_provider.base_url
        )
        username = json.loads(response.content)['userPrincipalName']

        try:
            self._session = self._session_type(
                username=username,
                token_type=rcont["token_type"],
                expires_in=rcont["expires_in"],
                scope_string=rcont["scope"],
                access_token=rcont["access_token"],
                client_id=self.client_id,
                auth_server_url=self._auth_token_url,
                redirect_uri=redirect_uri,
                refresh_token=rcont["refresh_token"] if "refresh_token" in rcont else None,
                client_secret=client_secret
            )
        except:
            raise Exception('response content:\n' + str(rcont))


def create_session(self, item=None):
    return ItemCreateSessionRequestBuilder(self.append_to_request_url("createUploadSession"), self._client, item=item)


def http_response_init(self, status, headers, content):
    self._status = status
    self._headers = headers
    self._content = content


@property
def session(self):
    return self._session


def logout(self):
    has_other_user = self._session.logout()
    if not has_other_user:
        self._session = None
    return has_other_user

# Overwrite the standard upload operation to use this one
AuthProvider.authenticate = authenticate
AuthProvider.session = session
AuthProvider.logout = logout
ItemRequestBuilder.create_session = create_session
HttpResponse.__init__ = http_response_init
