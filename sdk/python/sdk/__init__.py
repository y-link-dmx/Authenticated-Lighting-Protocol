from alpine_protocol import sdk as bindings_sdk

__all__ = ["SdkClient"]

class SdkClient:
    def __init__(self, client: bindings_sdk.AlpineClient):
        self._client = client

    @classmethod
    def connect(cls, *args, **kwargs):
        return cls(bindings_sdk.AlpineClient.connect(*args, **kwargs))

    def send_frame(self, frame):
        return self._client.send_frame(frame)
