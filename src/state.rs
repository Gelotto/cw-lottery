use crate::models::lottery::Lottery;
use crate::models::player::Player;
use crate::models::round::Round;
use crate::models::royalties::Claim;
use crate::models::ticket_order::TicketOrder;
use crate::models::winner::Winner;
use crate::msg::InstantiateMsg;
use crate::random::seed;
use crate::{error::ContractError, models::incentive::Incentive};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Storage};
use cw_storage_plus::{Item, Map};

pub const LOTTERY: Item<Lottery> = Item::new("lottery");
pub const ROUNDS: Map<u32, Round> = Map::new("rounds");
pub const INCENTIVES: Map<u32, Vec<Incentive>> = Map::new("incentives");
pub const PLAYERS: Map<(u32, Addr), Player> = Map::new("player");
pub const WINNERS: Map<(u32, Addr), Winner> = Map::new("winners");
pub const ORDERS: Map<(u32, u32), TicketOrder> = Map::new("orders");
pub const CLAIMS: Map<Addr, Claim> = Map::new("claims");
pub const SEED: Item<String> = Item::new("seed");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<Lottery, ContractError> {
  let lottery = Lottery::instantiate(info, msg)?;
  let round = Round::new(env.block.time, lottery.is_active(), 0);

  LOTTERY.save(deps.storage, &lottery)?;
  ROUNDS.save(deps.storage, 0, &round)?;
  SEED.save(deps.storage, &seed::init(&info.sender, env.block.height))?;

  Ok(lottery)
}

pub fn load_round(
  storage: &mut dyn Storage,
  lottery: &Lottery,
  round_index: Option<u32>,
) -> Result<Round, ContractError> {
  Ok(ROUNDS.load(storage, round_index.unwrap_or(lottery.rounds.index))?)
}

pub fn load_player(
  storage: &mut dyn Storage,
  round_index: u32,
  wallet_address: &Addr,
) -> Result<Player, ContractError> {
  PLAYERS
    .load(storage, (round_index, wallet_address.clone()))
    .or(Err(ContractError::PlayerNotFound {}))
}

pub fn remove_player_from_round(
  storage: &mut dyn Storage,
  player: &Player,
  round_index: u32,
  round: &mut Round,
) -> Result<(), ContractError> {
  // decrement counts
  round.counts.wallets -= 1;
  round.counts.tickets -= player.ticket_count;
  round.counts.orders -= player.order_indices.len() as u32;
  // remove the player
  PLAYERS.remove(storage, (round_index, player.wallet.clone()));
  // remove each order
  for order_index in player.order_indices.iter() {
    ORDERS.remove(storage, (round_index, *order_index));
  }
  Ok(())
}
