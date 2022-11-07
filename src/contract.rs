#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query;
use crate::{execute, state};
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-contract-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  state::initialize(deps, &env, &info, &msg)?;
  Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::BuyTickets {
      count,
      message,
      is_public,
    } => execute::buy_tickets(deps, env, info, count, message, is_public.unwrap_or(false)),
    ExecuteMsg::AddIncentives { rewards } => execute::add_incentives(deps, env, info, &rewards),
    ExecuteMsg::ClaimRewards {} => execute::claim_rewards(deps, env, info),
    ExecuteMsg::TerminateRound {} => execute::terminate_round(deps, env, info),
    ExecuteMsg::IssueRefund { round, recipient } => {
      execute::issue_refund(deps, env, info, round, &recipient)
    },
  }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> Result<Binary, ContractError> {
  let result = match msg {
    QueryMsg::GetRound {
      index,
      players,
      winners,
      orders,
    } => to_binary(&query::get_round::get_round(
      deps, index, players, winners, orders,
    )?),
  }?;
  Ok(result)
}
