use crate::{
  error::ContractError,
  models::{lottery::Lottery, round::Token},
  state::LOTTERY,
  utils::{build_cw20_transfer_msg, build_native_send_msg},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

/// Winners and Royalty Recipients claim rewards with this method.
pub fn claim_rewards(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> Result<Response, ContractError> {
  let lottery: Lottery = LOTTERY.load(deps.storage)?;

  // TODO: transfer winnings to sender
  // TODO: transfer any additional incentives to sender
  // TODO: update Claim

  let response = Response::new().add_attributes(vec![attr("action", "claim_rewards")]);
  Ok(response)
}
