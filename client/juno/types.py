
from dataclasses import dataclass
from decimal import Decimal
from typing import Optional, Union

Number = Union[Decimal, int, str]

@dataclass
class Balance:
    denom: str
    amount: Decimal

@dataclass
class Key:
    type: str
    name: str
    address: str
    pubkey: dict
    mnemonic: Optional[str]
