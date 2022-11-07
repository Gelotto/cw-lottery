use crate::{
  error::ContractError,
  models::{
    lottery::Lottery,
    round::{RoyaltyRecipient, Token},
  },
  state::{load_round, LOTTERY},
  utils::{build_cw20_transfer_msg, build_native_send_msg},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

/// Winners and Royalty Recipients claim rewards with this method.
pub fn terminate_round(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> Result<Response, ContractError> {
  let mut lottery: Lottery = LOTTERY.load(deps.storage)?;
  let mut round = load_round(deps.storage, &lottery, None)?;
  let config = lottery.get_config().clone();

  let royalties: Vec<RoyaltyRecipient> = if round.should_end(&config, env.block.time) {
    if lottery.rounds.index < lottery.rounds.count {
      lottery.end_round(deps.storage, &env, &info, &config, &mut round)?
    } else {
      vec![]
    }
  } else {
    vec![]
  };

  let response = Response::new().add_attributes(vec![attr("action", "terminate_round")]);

  // TODO: create CW messages for royalty transfers

  Ok(response)
}
