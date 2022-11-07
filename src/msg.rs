use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::{incentive::Reward, round::Config};

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub name: Option<String>,
  pub rounds: InitialRounds,
  pub tournament: Option<bool>,
  pub activate: Option<bool>,
}

/// Initial contract state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitialRounds {
  pub configs: Vec<Config>,
  pub count: u32,
}

/// Executable contract endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  BuyTickets {
    count: u32,
    message: Option<String>,
    is_public: Option<bool>,
  },
  AddIncentives {
    rewards: Vec<Reward>,
  },
  IssueRefund {
    round: u32,
    recipient: Addr,
  },
  ClaimRewards {},
  TerminateRound {},
}

/// Custom contract query endpoints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  GetRound {
    index: u32,
    players: Option<bool>,
    winners: Option<bool>,
    orders: Option<bool>,
  },
}
