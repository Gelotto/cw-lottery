use crate::{
  error::ContractError,
  models::{lottery::Lottery, round::Token},
  state::{load_player, load_round, remove_player_from_round, LOTTERY},
  utils::{build_cw20_transfer_msg, build_native_send_msg},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response};

/// Lottery owner can issue a refund to a specific wallet in a specific round
/// while the round is either active or canceled.
pub fn issue_refund(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  round_index: u32,
  recipient: &Addr,
) -> Result<Response, ContractError> {
  let lottery: Lottery = LOTTERY.load(deps.storage)?;
  let mut round = load_round(deps.storage, &lottery, Some(round_index))?;

  // only allow admin to issue a refund
  if info.sender != lottery.owner {
    return Err(ContractError::NotAuthorized {});
  }

  // only allow refunds for canceled rounds
  if !round.is_canceled() {
    return Err(ContractError::Forbidden {});
  }

  // get the refund-recipient's Player data for the specified round
  let player = load_player(deps.storage, round_index, recipient)?;

  // clear this wallet and its orders from Round state
  remove_player_from_round(deps.storage, &player, round_index, &mut round)?;

  // compute amount owed to refund-claimer
  let round_config = lottery.get_config();
  let refund_amount = player.amount_spent_in_round(round_config);

  // initialize response
  let response = Response::new().add_attributes(vec![
    attr("action", "issue_refund"),
    attr("recipient", recipient.to_string()),
    attr("round", round_index.to_string()),
    attr("amount", refund_amount.to_string()),
  ]);

  // add cw20 or native coin transfer message to response
  Ok(match round_config.token.clone() {
    Token::Native { denom } => {
      response.add_message(build_native_send_msg(&info.sender, &denom, refund_amount)?)
    },
    Token::Cw20 {
      address: cw20_token_address,
    } => response.add_submessage(build_cw20_transfer_msg(
      &env.contract.address,
      &info.sender,
      &cw20_token_address,
      refund_amount,
    )?),
  })
}
