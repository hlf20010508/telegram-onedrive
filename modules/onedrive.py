"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from onedrivesdk import HttpProvider, AuthProvider, OneDriveClient
from onedrivesdk.options import HeaderOption
from onedrivesdk.error import OneDriveError, ErrorCode
from onedrivesdk.model.upload_session import UploadSession
from onedrivesdk.model.item import Item
from onedrivesdk.request.item_create_session import ItemCreateSessionRequestBuilder
from onedrivesdk.request.item_request_builder import ItemRequestBuilder
from onedrivesdk.request_builder_base import RequestBuilderBase
from onedrivesdk.request_base import RequestBase
from onedrivesdk.http_response import HttpResponse
import json
import asyncio
import time


def authenticate_request(self, request):
    if self._session is None:
        raise RuntimeError("""Session must be authenticated 
            before applying authentication to a request.""")

    if self._session.is_expired() and 'offline_access' in self.scopes:
        self.refresh_token()
        self.save_session(path='session/onedrive.session')

    request.append_option(
        HeaderOption("Authorization",
                        "bearer {}".format(self._session.access_token)))


def create_session(self, item=None):
    return ItemCreateSessionRequestBuilder(self.append_to_request_url("createUploadSession"), self._client, item=item)


def http_response_init(self, status, headers, content):
    self._status = status
    self._headers = headers
    self._content = content


class Onedrive:
    def __init__(self, client_id, client_secret, redirect_uri, remote_root_path):
        api_base_url = "https://graph.microsoft.com/v1.0/"
        auth_server_url = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"

        scopes = ["offline_access", "Files.ReadWrite"]

        http_provider = HttpProvider()
        auth_provider = AuthProvider(
            http_provider=http_provider,
            client_id=client_id,
            scopes=scopes,
            auth_server_url=auth_server_url
        )

        self.session_path = 'session/onedrive.session'
        self.remote_root_path = remote_root_path
        self.client_secret = client_secret
        self.redirect_uri = redirect_uri
        self.client = OneDriveClient(
            api_base_url,
            auth_provider,
            http_provider
        )

    def get_auth_url(self):
        return self.client.auth_provider.get_auth_url(self.redirect_uri)

    def auth(self, auth_code):
        self.client.auth_provider.authenticate(
            auth_code,
            self.redirect_uri,
            self.client_secret
        )
        self.save_session()
    
    def save_session(self):
        self.client.auth_provider.save_session(path=self.session_path)

    def load_session(self):
        self.client.auth_provider.load_session(path=self.session_path)

    def stream_upload(self, buffer, name):
        request = self.client.item(path=self.remote_root_path).children[name].content.request()
        request.method = 'PUT'
        request.send(data=buffer)

    def multipart_upload_session_builder(self, name):
        item = Item({})
        session = self.client.item(path=self.remote_root_path).children[name].create_session(item).post()
        return session
    
    def multipart_uploader(self, session, total_length):
        return ItemUploadFragmentBuilder(session.upload_url, self.client, total_length)
    
    async def multipart_upload(self, uploader, buffer, offset, part_size):
        tries = 0
        while True:
            try:
                tries += 1
                uploader.post(offset, part_size, buffer)
            except OneDriveError as exc:
                if exc.status_code in (408, 500, 502, 503, 504) and tries < 5:
                    await asyncio.sleep(0.1)
                    continue
                elif exc.status_code == 416:
                    # Fragment already received
                    break
                elif exc.status_code == 401:
                    self._client.auth_provider.refresh_token()
                    continue
                else:
                    raise exc
            except ValueError:
                # Swallow value errors (usually JSON error) and try again.
                continue
            break  # while True
    
    def upload_from_url(self, url, name):
        opts = [
            HeaderOption('Prefer', 'respond-async'),
        ]

        request = self.client.item(path=self.remote_root_path).children.request(options=opts)
        request.content_type = 'application/json'
        request.method = 'POST'

        data = {
            "@microsoft.graph.sourceUrl": url,
            "name": name,
            "file": {}
        }

        tries = 0
        while tries < 5:
            response = request.send(content=data)
            if response.status == 202:
                progress_url = response.headers['Location']
                return progress_url
            else:
                tries += 1
                time.sleep(0.1)
                continue

        response_dict = {
            'Status': response.status,
            'Headers': response.headers,
            'Content': response.content
        }
        raise Exception('upload from url response error: ' + str(response_dict))
    
    def upload_from_url_progress(self, url):        
        tries = 0
        while tries < 5:
            response = self.client.http_provider.send(
                method="GET",
                headers={},
                url=url
            )
            if response.status >= 200 and response.status < 300:
                break
            else:
                tries += 1
                time.sleep(0.1)
                continue
        try:
            response._content = json.loads(response.content)
        except:
            response._content = {
                "error": {
                    "code": ErrorCode.Malformed,
                    "message": "The following invalid JSON was returned:\n%s" % response.content
                }
            }
        return response


class ItemUploadFragment(RequestBase):
    def __init__(self, request_url, client, options, buffer):
        super(ItemUploadFragment, self).__init__(request_url, client, options)
        self.method = "PUT"
        self._buffer = buffer

    def post(self):
        """Sends the POST request

        Returns:
            :class:`UploadSession<onedrivesdk.model.upload_session.UploadSession>`:
                The resulting entity from the operation
        """
        entity = UploadSession(json.loads(self.send(data=self._buffer).content))
        return entity


class ItemUploadFragmentBuilder(RequestBuilderBase):
    def __init__(self, request_url, client, total_length):
        super(ItemUploadFragmentBuilder, self).__init__(request_url, client)
        self._method_options = {}
        self._total_length = total_length

    def __enter__(self):
        return self

    def __exit__(self, type, value, traceback):
        self._buffer.close()

    def request(self, begin, length, buffer, options=None):
        """Builds the request for the ItemUploadFragment

        Args:
            begin (int): First byte in range to be uploaded
            length (int): Number of bytes in range to be uploaded
            options (list of :class:`Option<onedrivesdk.options.Option>`):
                Default to None, list of options to include in the request

        Returns:
            :class:`ItemUploadFragment<onedrivesdk.request.item_upload_fragment.ItemUploadFragment>`:
                The request
        """
        if not (options is None or len(options) == 0):
            opts = options.copy()
        else:
            opts = []

        self.content_type = "application/octet-stream"

        opts.append(
            HeaderOption(
                "Content-Range",
                "bytes %d-%d/%d" % (begin, begin + length - 1, self._total_length),
            )
        )
        opts.append(HeaderOption("Content-Length", str(length)))

        req = ItemUploadFragment(self._request_url, self._client, opts, buffer)
        return req

    def post(self, begin, length, buffer, options=None):
        """Sends the POST request

        Returns:
            :class:`UploadedFragment<onedrivesdk.model.uploaded_fragment.UploadedFragment>`:
            The resulting UploadSession from the operation
        """
        return self.request(begin, length, buffer, options).post()


# Overwrite the standard upload operation to use this one
AuthProvider.authenticate_request = authenticate_request
ItemRequestBuilder.create_session = create_session
HttpResponse.__init__ = http_response_init