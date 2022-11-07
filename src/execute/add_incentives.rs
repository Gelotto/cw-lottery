use crate::{
  error::ContractError,
  models::{
    incentive::{Incentive, Reward},
    lottery::Lottery,
    round::Token,
  },
  state::{load_round, INCENTIVES, LOTTERY},
  utils::{build_cw20_transfer_msg, build_native_send_msg, verify_cw20_funds, verify_native_funds},
};
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg};

/// Add additional native tokens, cw20 tokens, NFT's or other assets
/// to the current active round's pot.
pub fn add_incentives(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  rewards: &Vec<Reward>,
) -> Result<Response, ContractError> {
  let lottery: Lottery = LOTTERY.load(deps.storage)?;
  let round = load_round(deps.storage, &lottery, None)?;

  // abort if the current round is not active
  if !round.is_active() {
    return Err(ContractError::NotActive {});
  }
  // abort if the incentive's rewards are empty
  if rewards.len() == 0 {
    return Err(ContractError::MissingRewards {});
  }

  // verify that the sender actually has sufficient balances of the
  // token types they're adding as incentives. at the same time, prepare
  // the required transfer messages to add to the response.
  let mut cw20_transfer_submsgs: Vec<SubMsg> = vec![];
  let mut bank_transfer_msgs: Vec<CosmosMsg> = vec![];
  for reward in rewards.iter() {
    if let Some(t) = reward.token.clone() {
      match t.token {
        Token::Cw20 {
          address: cw20_address,
        } => {
          verify_cw20_funds(&deps, &info.sender, t.amount, &cw20_address)?;
          cw20_transfer_submsgs.push(build_cw20_transfer_msg(
            &info.sender,
            &env.contract.address,
            &cw20_address,
            t.amount,
          )?);
        },
        Token::Native { denom } => {
          verify_native_funds(&info.funds, t.amount, &denom)?;
          bank_transfer_msgs.push(build_native_send_msg(
            &env.contract.address,
            &denom,
            t.amount,
          )?);
        },
      }
    }
  }

  // persist the rewards in a new Incentive, adding it to
  // a vec associated with current round.
  INCENTIVES.update(
    deps.storage,
    lottery.rounds.index,
    |some_incentives| -> Result<Vec<Incentive>, ContractError> {
      if let Some(mut incentives) = some_incentives {
        incentives.push(Incentive {
          source: info.sender.clone(),
          rewards: rewards.clone(),
        });
        Ok(incentives)
      } else {
        Ok(vec![Incentive {
          source: info.sender.clone(),
          rewards: rewards.clone(),
        }])
      }
    },
  )?;

  Ok(
    Response::new()
      .add_attributes(vec![
        attr("action", "add_incentives"),
        attr("source", info.sender.to_string()),
      ])
      .add_messages(bank_transfer_msgs)
      .add_submessages(cw20_transfer_submsgs),
  )
}
