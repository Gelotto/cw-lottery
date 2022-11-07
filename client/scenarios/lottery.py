from juno.util import pretty
from juno.client import Client, Wallet, Contract
from juno.scenario import Scenario


class LotteryScenario(Scenario):
    def query_round(self, contract: Contract, index: int) -> dict:
        return contract.query(
            'get_round',
            {'index': index, 'players': True,
             'winners': True, 'orders': True}
        )

    def buy_tickets(self, contract: Contract, player: Wallet, count: int, funds) -> dict:
        return contract.execute(
            'buy_tickets', {'ticket_count': count}, funds=funds, sender=player,
        )

    def run(self, client: Client):
        contract = client.instantiate(25, {
            "name": "Lottery-v2",
            "activate": True,
            "rounds": {
                "count": 4,
                "configs": [
                    {
                        "name": "Round-1",
                        "targets": {
                            "funding_level": "10000",
                            "duration_minutes": 360,
                        },
                        "winners": {
                            "distinct_wallets": True,
                            "winner_count": {
                                "fixed": 1
                            }
                        },
                        "token": {
                            "ibc": {"denom": "ujunox"}
                        },
                        "ticket_price": "1000",
                        "max_tickets_per_wallet": 5,
                        "royalties": []
                    }
                ]
            }
        })

        dan = client.wallet('dan').set_balance(1e6)
        kat = client.wallet('kat').set_balance(1e6)

        pretty(self.buy_tickets(contract, dan, 1, funds=1000))
        pretty(self.buy_tickets(contract, kat, 5, funds=5000))
        pretty(self.buy_tickets(contract, dan, 4, funds=4000))

        pretty(self.query_round(contract, 0))


if __name__ == '__main__':
    scenario = LotteryScenario()
    scenario.start()
