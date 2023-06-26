import onedrivesdk
import os

class Onedrive:
    def __init__(self, client_id, client_secret, redirect_uri, remote_root_path):
        api_base_url = "https://api.onedrive.com/v1.0/"
        scopes = ["wl.signin", "wl.offline_access", "onedrive.readwrite"]

        http_provider = onedrivesdk.HttpProvider()
        auth_provider = onedrivesdk.AuthProvider(
            http_provider=http_provider, client_id=client_id, scopes=scopes
        )

        self.remote_root_path = remote_root_path
        self.client_secret = client_secret
        self.redirect_uri = redirect_uri
        self.client = onedrivesdk.OneDriveClient(
            api_base_url, auth_provider, http_provider
        )

    def get_auth_url(self):
        return self.client.auth_provider.get_auth_url(self.redirect_uri)

    def auth(self, auth_code):
        self.client.auth_provider.authenticate(
            auth_code, self.redirect_uri, self.client_secret
        )

    def upload(self, file_path):
        name = file_path.split("/")[-1]
        self.client.item(path=self.remote_root_path).children[name].upload(file_path)
        return os.path.join(self.remote_root_path, name)
