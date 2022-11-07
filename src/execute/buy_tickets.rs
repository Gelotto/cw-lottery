use crate::{
  error::ContractError,
  models::{
    lottery::Lottery,
    player::Player,
    round::{Config, Round, RoyaltyRecipient, Token, WinnerSelectionMethod},
    ticket_order::TicketOrder,
  },
  random::seed,
  state::{LOTTERY, ORDERS, PLAYERS, ROUNDS, SEED},
  utils::{verify_cw20_funds, verify_native_funds},
};
use cosmwasm_std::{
  attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg,
  Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

pub fn buy_tickets(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  order_ticket_count: u32,
  message: Option<String>,
  is_public: bool,
) -> Result<Response, ContractError> {
  let buyer = &info.sender;
  let mut lottery: Lottery = LOTTERY.load(deps.storage)?;
  let config_index = lottery.get_config_index();
  let config = lottery.get_config().clone();
  let round_index = lottery.rounds.index;
  let mut round = ROUNDS.load(deps.storage, round_index)?;

  // abort if this round is not longer active
  if !round.is_active() {
    return Err(ContractError::InactiveRound {});
  }

  // unless this round has the initial config, abort if the sender didn't
  // participate in the previous round and this is a tournament
  if config_index > 0 {
    if !PLAYERS.has(deps.storage, (round_index - 1, buyer.clone())) {
      return Err(ContractError::Forbidden {});
    }
  }

  let order_index = round.counts.orders;

  // get or create a player record
  let mut player = match PLAYERS.may_load(deps.storage, (round_index, buyer.clone()))? {
    Some(player) => player,
    None => Player {
      wallet: buyer.clone(),
      order_indices: vec![],
      ticket_count: 0,
    },
  };
  // update round metadata
  round.counts.tickets += order_ticket_count;
  round.counts.drawings = get_updated_winner_count(&config, &round);
  round.counts.orders += 1;

  if player.ticket_count == 0 {
    round.counts.wallets += 1;
  }
  // increment the player's total ticket count in the current round
  player.ticket_count += order_ticket_count;
  player.order_indices.push(order_index);

  // abort if the player's new total ticket count exceeds the max number of
  // tickets allowed per wallet, if defined.
  if let Some(max_tickets_per_wallet) = config.max_tickets_per_wallet {
    if player.ticket_count > max_tickets_per_wallet {
      return Err(ContractError::TooManyTickets {
        max_tickets_per_wallet,
      });
    }
  }

  // autosent_royalties is populated only if this buy_tickets execution
  // results in the completion of the round. background: a claims record is
  // upserted for all non-autosent royalty recipients; however, for all autosent
  // recipients, a transfer is performed in this tx.
  let royalties: Vec<RoyaltyRecipient> = if round.should_end(&config, env.block.time) {
    if lottery.rounds.index < lottery.rounds.count {
      lottery.end_round(deps.storage, &env, &info, &config, &mut round)?
    } else {
      vec![]
    }
  } else {
    vec![]
  };

  // persist changes to state
  SEED.update(deps.storage, |seed| -> Result<String, ContractError> {
    Ok(seed::update(
      &seed,
      &info.sender,
      order_ticket_count,
      env.block.height,
      &message,
    ))
  })?;
  PLAYERS.save(deps.storage, (round_index, buyer.clone()), &player)?;
  ROUNDS.save(deps.storage, lottery.rounds.index, &round)?;
  ORDERS.save(
    deps.storage,
    (lottery.rounds.index, order_index),
    &TicketOrder {
      wallet: info.sender.clone(),
      ticket_count: order_ticket_count,
      message,
      is_public,
    },
  )?;

  // compute total price of the ticket order
  let total_cost = config.ticket_price * Uint128::from(order_ticket_count);

  // generate a response with a msg or submsg that performs the required
  // transfer from sender to this contract.
  Ok(match config.token.clone() {
    Token::Native { denom } => build_response_with_ibc_transfer(
      &info,
      &env,
      &royalties,
      &denom,
      order_ticket_count,
      total_cost,
    )?,
    Token::Cw20 { address } => build_response_with_cw20_transfer(
      &deps,
      &info,
      &env,
      &royalties,
      &address,
      order_ticket_count,
      total_cost,
    )?,
  })
}

fn get_updated_winner_count(
  config: &Config,
  round: &Round,
) -> u32 {
  match config.selection.method.clone() {
    WinnerSelectionMethod::Percent { pct, max } => {
      let mut winner_count = (((pct as u32) * round.counts.wallets) / 100u32).max(1);
      let max_winner_count = max.unwrap_or(0);
      if max_winner_count > 0 {
        winner_count = round.counts.drawings.min(max_winner_count);
      }
      winner_count
    },
    WinnerSelectionMethod::Fixed(split) => split.len() as u32,
  }
}

fn build_response_with_cw20_transfer(
  deps: &DepsMut,
  info: &MessageInfo,
  env: &Env,
  royalties: &Vec<RoyaltyRecipient>,
  cw20_token_address: &Addr,
  ticket_count: u32,
  amount: Uint128,
) -> Result<Response, ContractError> {
  verify_cw20_funds(&deps, &info.sender, amount, &cw20_token_address)?;
  // perform CW20 transfer from sender to contract.  note that the cw20
  // token allowance for this contract must be set.
  let execute_msg = WasmMsg::Execute {
    contract_addr: cw20_token_address.clone().into(),
    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
      owner: info.sender.clone().into(),
      recipient: env.contract.address.clone().into(),
      amount,
    })?,
    funds: vec![],
  };
  // royalties will only be non-empty if this is the end of the round
  let mut send_royalty_submsgs: Vec<SubMsg> = Vec::with_capacity(royalties.len());
  for royalty in royalties.iter() {
    let royalty_amount = (Uint128::from(royalty.pct) * amount) / Uint128::from(100u8);
    send_royalty_submsgs.push(SubMsg::new(WasmMsg::Execute {
      contract_addr: cw20_token_address.clone().into(),
      msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.clone().into(),
        recipient: env.contract.address.clone().into(),
        amount: royalty_amount,
      })?,
      funds: vec![],
    }));
  }

  Ok(
    Response::new()
      .add_submessage(SubMsg::new(execute_msg))
      .add_submessages(send_royalty_submsgs)
      .add_attributes(vec![
        attr("action", "buy_tickets"),
        attr("ticket_count", ticket_count.to_string()),
      ]),
  )
}

fn build_response_with_ibc_transfer(
  info: &MessageInfo,
  env: &Env,
  royalties: &Vec<RoyaltyRecipient>,
  ibc_denom: &String,
  ticket_count: u32,
  amount: Uint128,
) -> Result<Response, ContractError> {
  verify_native_funds(&info.funds, amount, ibc_denom)?;
  // Perform transfer of IBC asset from sender to contract.
  let send_payment_message = CosmosMsg::Bank(BankMsg::Send {
    to_address: env.contract.address.clone().into_string(),
    amount: vec![Coin::new(amount.u128(), ibc_denom)],
  });
  // royalties will only be non-empty if this is the end of the round
  let send_royalty_messages: Vec<CosmosMsg> = royalties
    .iter()
    .map(|x| {
      let royalty_amount = (Uint128::from(x.pct) * amount) / Uint128::from(100u8);
      CosmosMsg::Bank(BankMsg::Send {
        to_address: env.contract.address.clone().into_string(),
        amount: vec![Coin::new(royalty_amount.u128(), ibc_denom)],
      })
    })
    .collect();

  Ok(
    Response::new()
      .add_message(send_payment_message)
      .add_messages(send_royalty_messages)
      .add_attributes(vec![
        attr("action", "buy_tickets"),
        attr("ticket_count", ticket_count.to_string()),
      ]),
  )
}
