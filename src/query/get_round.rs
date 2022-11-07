use cosmwasm_std::{Addr, Deps, Order, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  error::ContractError,
  models::{
    lottery::Lottery,
    player::Player,
    round::{Config, Counts, RoundStatus},
    ticket_order::TicketOrder,
    winner::Winner,
  },
  state::{LOTTERY, ORDERS, PLAYERS, ROUNDS, WINNERS},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetRoundResponse {
  config: Config,
  players: Option<Vec<Player>>,
  winners: Option<Vec<Winner>>,
  orders: Option<Vec<TicketOrder>>,
  status: RoundStatus,
  counts: Counts,
  started_at: Option<Timestamp>,
  ended_by: Option<Addr>,
}

pub fn get_round(
  deps: Deps,
  round_index: u32,
  include_players: Option<bool>,
  include_winners: Option<bool>,
  include_orders: Option<bool>,
) -> Result<GetRoundResponse, ContractError> {
  let lottery: Lottery = LOTTERY.load(deps.storage)?;

  if round_index > lottery.rounds.index {
    return Err(ContractError::RoundNotFound {});
  }

  let config = &lottery.rounds.configs[(round_index as usize) % (lottery.rounds.configs.len())];
  let round = ROUNDS.load(deps.storage, round_index)?;

  let players: Option<Vec<Player>> = if include_players.unwrap_or(false) {
    Some(
      PLAYERS
        .prefix(round_index)
        .range(deps.storage, None, None, Order::Descending)
        .map(|entry| entry.unwrap().1)
        .collect(),
    )
  } else {
    None
  };

  let winners: Option<Vec<Winner>> = if include_winners.unwrap_or(false) {
    Some(
      WINNERS
        .prefix(round_index)
        .range(deps.storage, None, None, Order::Descending)
        .map(|entry| entry.unwrap().1)
        .collect(),
    )
  } else {
    None
  };

  let orders: Option<Vec<TicketOrder>> = if include_orders.unwrap_or(false) {
    Some(
      ORDERS
        .prefix(round_index)
        .range(deps.storage, None, None, Order::Descending)
        .map(|entry| entry.unwrap().1)
        .collect(),
    )
  } else {
    None
  };

  Ok(GetRoundResponse {
    config: config.clone(),
    status: round.status,
    started_at: round.started_at,
    ended_by: round.ended_by,
    counts: round.counts,
    players,
    winners,
    orders,
  })
}
