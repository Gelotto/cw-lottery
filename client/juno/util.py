from dataclasses import dataclass
import json
import re
import subprocess as sp

from decimal import Decimal, InvalidOperation
from base64 import decodebytes
from typing import List

from .types import Key, Balance

class ContractMsgError(Exception):
    def __init__(self, msg_index: int, error: str):
        super().__init__('contract msg error')
        self.msg_index = msg_index
        self.error = error


def get_wallet_address(name: str) -> str:
    data = run(f'''junod keys show {name} --output json''')
    return data['address']

def get_balances(address: str) -> List[Balance]:
    data = run(f'''junod query bank balances {address} --output json''')
    for balance in data['balances']:
        balance['amount'] = Decimal(balance['amount'])
    return data

def b64_decode(data: dict) -> dict:
    events = data.get("events", [])
    for event in events:
        event_attrs = event.get("attributes", [])
        for attr in event_attrs:
            attr["key"] = decodebytes(attr["key"].encode()).decode()
            attr["value"] = decodebytes(attr["value"].encode()).decode()
    return data

def get_balance_amount(address: str, denom: str) -> Decimal:
    data = get_balances(address)
    for balance in data['balances']:
        if balance['denom'] == denom:
            return Decimal(balance['amount'])
    return Decimal(0)

def get_or_create_key(name: str) -> Key:
    data = run(f'''junod keys show {name} --output json''')
    if not data:
        data = run(f'''junod keys add {name} --output json''')
    data['pubkey'] = json.loads(data['pubkey'])
    data.setdefault('mnemonic', None)
    return Key(**data)

def run(cmd: str, decode=False, echo=False) -> dict:
    cmd = re.sub(f'\s+', ' ', cmd).strip()
    if echo:
        print(cmd)
    proc = sp.run(cmd, capture_output=True, shell=True)
    if proc.stderr:
        err = proc.stderr.decode()
        print(err)
        error_info_match = re.search(r'message index: (\d+): (\w+)', err)
        if error_info_match:
            (error, msg_index) = error_info_match.groups()
            raise ContractMsgError(msg_index, error)
    data = json.loads(proc.stdout or '{}')
    return b64_decode(data) if decode else data


def pretty(data: dict):
    print(json.dumps(data, indent=2, sort_keys=True))