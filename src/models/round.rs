use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoundStatus {
  Pending,
  Active,
  Closed,
  Complete,
  Canceled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Token {
  Native { denom: String },
  Cw20 { address: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyRecipient {
  pub address: Addr,
  pub autosend: Option<bool>,
  pub pct: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Targets {
  pub funding_level: Option<Uint128>,
  pub duration_minutes: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WinnerSelectionMethod {
  Percent { pct: u8, max: Option<u32> },
  Fixed(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnerSelection {
  pub method: WinnerSelectionMethod,
  pub with_replacement: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
  pub name: Option<String>,
  pub targets: Targets,
  pub selection: WinnerSelection,
  pub token: Token,
  pub ticket_price: Uint128,
  pub max_tickets_per_wallet: Option<u32>,
  pub royalties: Vec<RoyaltyRecipient>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Counts {
  pub drawings: u32,
  pub wallets: u32,
  pub tickets: u32,
  pub orders: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Round {
  pub status: RoundStatus,
  pub counts: Counts,
  pub started_at: Option<Timestamp>,
  pub ended_by: Option<Addr>,
  pub index: u32,
}

impl Round {
  pub fn new(
    started_at: Timestamp,
    is_active: bool,
    index: u32,
  ) -> Self {
    Self {
      ended_by: None,
      started_at: if is_active { Some(started_at) } else { None },
      index,
      status: if is_active {
        RoundStatus::Active
      } else {
        RoundStatus::Pending
      },
      counts: Counts {
        drawings: 0,
        wallets: 0,
        tickets: 0,
        orders: 0,
      },
    }
  }

  pub fn is_active(&self) -> bool {
    self.status == RoundStatus::Active
  }

  pub fn is_canceled(&self) -> bool {
    self.status == RoundStatus::Canceled
  }

  pub fn should_end(
    &self,
    config: &Config,
    block_time: Timestamp,
  ) -> bool {
    if let Some(funding_level) = config.targets.funding_level {
      Uint128::from(self.counts.tickets) * config.ticket_price >= funding_level
    } else if let Some(minute_duration) = config.targets.duration_minutes {
      if let Some(started_at) = self.started_at {
        block_time >= started_at.plus_seconds((minute_duration as u64) * 60)
      } else {
        false
      }
    } else {
      false
    }
  }

  pub fn get_pot_size(
    &self,
    config: &Config,
  ) -> Uint128 {
    Uint128::from(self.counts.tickets) * Uint128::from(config.ticket_price)
  }

  pub fn get_total_royalty_amount(
    &self,
    config: &Config,
    total: Uint128,
  ) -> Uint128 {
    config
      .royalties
      .iter()
      .map(|r| total * Uint128::from(r.pct) / Uint128::from(100u32))
      .sum()
  }
}
