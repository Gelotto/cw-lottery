use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::round::Token;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAmount {
  // the token received as a reward
  pub token: Token,
  pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
  // the token received as a reward
  pub token: Option<TokenAmount>,

  // the position who receives the reward (1st place, 2nd, etc.)
  pub position: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Incentive {
  // the wallet address that added the incentive
  pub source: Addr,

  // list of rewards to be received by winners as the incentive
  pub rewards: Vec<Reward>,
}
