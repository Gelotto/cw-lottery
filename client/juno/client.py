from ast import Add
import json

from datetime import datetime
from decimal import Decimal
from typing import List, Union

from .types import Balance, Key, Number
from .util import get_balance_amount, get_or_create_key, run

Addressable = Union['Wallet', str]


class Contract:
    def __init__(self, client: 'Client', address: str):
        self.client = client
        self.address = address

    def execute(self, method: str, params: dict = {}, funds=None, sender: Addressable = None) -> dict:
        return self.client.execute(self.address, method, params, funds=funds, sender=sender)

    def query(self, method: str, params: dict = {}) -> dict:
        return self.client.query(self.address, method, params)


class Wallet:
    def __init__(self, client: 'Client', key: Key):
        self.client = client
        self.key = key

    @property
    def address(self) -> str:
        return self.key.address

    @property
    def name(self) -> str:
        return self.key.name

    def set_balance(self, amount: Number) -> 'Wallet':
        balance = self.get_balance()
        amount = Decimal(amount)
        if balance < amount:
            self.client.airdrop(self.address, amount - balance)
        if balance > amount:
            self.client.reclaim(self.address, balance - amount)
        return self

    def get_balances(self) -> List[Balance]:
        return self.client.get_balances(self.address)

    def get_balance(self, denom: str = None) -> Decimal:
        return get_balance_amount(self.address, denom or self.client.denom)


class Client:
    def __init__(self, network: str = 'devnet', sender: Union[str, Wallet] = None, echo=True):
        if network == 'testnet':
            self.node = "https://rpc.uni.juno.deuslabs.fi:443"
            self.chain_id = 'uni-3'
            self.denom = 'ujunox'
        elif network == 'mainnet':
            self.node = "https://rpc-juno.itastakers.com",
            self.chain_id = 'juno-1'
            self.denom = 'ujuno'
        else:
            self.node = "http://localhost:26657"
            self.chain_id = 'testing'
            self.denom = 'ujunox'

        self.faucet = 'juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y'
        self.default_sender = sender or 'juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y'
        self.gas_price = f'0.02{self.denom}'
        self.gas_adjustment = '1.3'
        self.echo = echo

    def wallet(self, name: str) -> Wallet:
        key = get_or_create_key(name)
        return Wallet(self, key)

    def contract(self, address: str) -> Contract:
        return Contract(self, address)

    @staticmethod
    def resolve_address(sender: Addressable) -> str:
        return sender if isinstance(sender, str) else sender.address

    def instantiate(self, code_id: int, msg: dict = {}, label: str = None, sender: Addressable = None) -> Contract:
        sender_addr = admin_addr = self.resolve_address(
            sender or self.default_sender)
        data = run(f'''
        junod tx wasm instantiate {code_id} '{json.dumps(msg)}'
            --node {self.node}
            --gas-prices {self.gas_price}
            --chain-id {self.chain_id}
            --from {sender_addr}
            --gas-adjustment {self.gas_adjustment}
            --label {label}-{datetime.now().timestamp()}
            --admin {admin_addr}
            --gas auto
            --broadcast-mode block
            --output json
            -y 
        '''.replace('\n', ' '), decode=True, echo=self.echo)
        for event in data['logs'][0]['events']:
            if event['type'] == 'instantiate':
                for attr in event['attributes']:
                    if attr['key'] == '_contract_address':
                        address = attr['value']
                        return Contract(self, address)

    def execute(self, address: str, method: str, params: dict = {}, funds=None, sender: Addressable = None) -> dict:
        sender_addr = self.resolve_address(sender or self.default_sender)
        execute_msg = json.dumps({method: params})
        return run(f'''
        junod tx wasm execute {address} '{execute_msg}'
            {('--amount ' + str(funds) + self.denom) if funds else ''}
            --node {self.node}
            --gas-prices {self.gas_price}
            --chain-id {self.chain_id}
            --from {sender_addr}
            --gas-adjustment {self.gas_adjustment}
            --gas auto
            --broadcast-mode block
            --output json
            -y 
        '''.replace('\n', ' '), decode=True, echo=self.echo)

    def query(self, address: str, method: str, params: dict = {}) -> dict:
        query_msg = json.dumps({method: params})
        return run(f'''
        junod query wasm contract-state smart {address} '{query_msg}'
            --node {self.node}
            --chain-id {self.chain_id}
            --output json
        '''.replace('\n', ' '), echo=self.echo)

    def send(self, sender: Addressable, recipient: Addressable, amount: Number) -> dict:
        sender_addr = self.resolve_address(sender)
        recipient_addr = self.resolve_address(recipient)
        amount = Decimal(amount)
        return run(f'''junod tx bank send {sender_addr} {recipient_addr} {amount}{self.denom}
            --node {self.node}
            --gas-prices {self.gas_price}
            --chain-id {self.chain_id}
            --from {sender_addr}
            --gas-adjustment {self.gas_adjustment}
            --gas auto
            --broadcast-mode block
            --output json
            -y
        '''.replace('\n', ' '), echo=self.echo)

    def airdrop(self, recipient: str, amount: Union[int, str, Decimal]) -> dict:
        return self.send(self.faucet, recipient, amount)

    def reclaim(self, address: str, amount: Number = None) -> dict:
        amount = amount if amount is not None else self.get_balance_amount(
            address, 'juno')
        return self.send(address, self.faucet, amount)
