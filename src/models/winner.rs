use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Winner {
  pub wallet: Addr,
  pub amount_total: Uint128,
  pub amount_claimed: Uint128,
  pub position: u16,
}
