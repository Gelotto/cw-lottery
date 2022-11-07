use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::round::Config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Player {
  pub wallet: Addr,
  pub ticket_count: u32,
  pub order_indices: Vec<u32>,
}

impl Player {
  pub fn amount_spent_in_round(
    &self,
    round_config: &Config,
  ) -> Uint128 {
    Uint128::from(self.ticket_count) * round_config.ticket_price
  }
}
