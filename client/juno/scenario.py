from .client import Client


class Scenario:
    def __init__(self, client: Client = None):
        self.client = client or Client()

    def start(self):
        self.run(self.client)

    def run(self, client: Client):
        pass