use cw_storage_plus::KeyDeserialize;
use std::{collections::HashSet, iter};

use crate::{
  error::ContractError,
  msg::InstantiateMsg,
  random::{pcg64_from_seed, seed},
  state::{CLAIMS, PLAYERS, ROUNDS, SEED},
  utils::apply_pct,
};

use super::{
  round::{Config, Round, RoundStatus, RoyaltyRecipient, WinnerSelectionMethod},
  royalties::Claim,
};
use cosmwasm_std::{Addr, Env, MessageInfo, Order, StdResult, Storage, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LotteryStatus {
  Pending,
  Active,
  Complete,
  Canceled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rounds {
  pub configs: Vec<Config>,
  pub index: u32,
  pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Lottery {
  pub owner: Addr,
  pub name: Option<String>,
  pub tournament: Option<bool>,
  pub status: LotteryStatus,
  pub rounds: Rounds,
}

impl Lottery {
  pub fn instantiate(
    info: &MessageInfo,
    msg: &InstantiateMsg,
  ) -> Result<Self, ContractError> {
    let lottery = Lottery {
      owner: info.sender.clone(),
      name: msg.name.clone(),
      tournament: msg.tournament,
      status: if msg.activate.unwrap_or(true) {
        LotteryStatus::Active
      } else {
        LotteryStatus::Pending
      },
      rounds: Rounds {
        configs: msg.rounds.configs.clone(),
        count: msg.rounds.count.max(msg.rounds.configs.len() as u32),
        index: 0,
      },
    };
    lottery.validate()?;
    Ok(lottery)
  }

  pub fn get_config_index(&self) -> usize {
    (self.rounds.index.clone() as usize) % self.rounds.configs.len()
  }

  pub fn get_config(&self) -> &Config {
    &self.rounds.configs[self.get_config_index()]
  }

  pub fn validate(&self) -> Result<(), ContractError> {
    if self.rounds.configs.len() == 0 {
      return Err(ContractError::ValidationError {
        reason: Some("lottery must have at least 1 round config".to_owned()),
      });
    }
    Ok(())
  }

  pub fn is_active(&self) -> bool {
    self.status == LotteryStatus::Active
  }

  pub fn end_round(
    &mut self,
    storage: &mut dyn Storage,
    env: &Env,
    info: &MessageInfo,
    config: &Config,
    round: &mut Round,
  ) -> Result<Vec<RoyaltyRecipient>, ContractError> {
    if !round.is_active() {
      return Err(ContractError::NotActive {});
    }

    if round.counts.wallets == 0 {
      return Ok(vec![]);
    } else if round.counts.wallets == 1 {
      Self::refund_tickets(storage, round)?;
      Self::refund_incentives(storage, round)?;
      return Ok(vec![]);
    }

    // end the current round
    round.ended_by = Some(info.sender.clone());
    round.status = RoundStatus::Complete;

    // increment the round index and create a the next active Round.
    // otherwise, mark the lottery completed as a whole
    let is_final_round = self.rounds.index == self.rounds.count - 1;
    if is_final_round {
      self.status = LotteryStatus::Complete;
    } else {
      let next_round_index = self.rounds.index + 1;
      // create the next round
      ROUNDS.save(
        storage,
        self.rounds.index + 1,
        &Round::new(env.block.time, true, next_round_index),
      )?;
    }

    // calculate claimable amounts
    let total_amount = round.get_pot_size(config);
    let total_royalty_amount = round.get_total_royalty_amount(config, total_amount);
    let total_winnings_amount = total_amount - total_royalty_amount;

    // get and save new PRNG seed
    let new_seed = seed::finalize(&SEED.load(storage)?, &info.sender, env.block.height);
    SEED.save(storage, &new_seed)?;

    // increment claimable amount for each non-autosent royalty recipient
    Self::upsert_royalty_claims(storage, config, &round)?;

    // randomly select the winners and increment their claim records
    Self::pick_winners_and_upsert_claims(
      storage,
      config,
      &round,
      total_winnings_amount,
      &new_seed,
    )?;

    // collect royalty recipients using autosend for the sake of forming
    // the required CW messages
    let royalties: Vec<RoyaltyRecipient> = config
      .royalties
      .iter()
      .filter(|x| x.autosend.unwrap_or(false))
      .map(|x| x.clone())
      .collect();

    Ok(royalties)
  }

  fn refund_tickets(
    storage: &mut dyn Storage,
    round: &Round,
  ) -> Result<(), ContractError> {
    Ok(())
  }

  fn refund_incentives(
    storage: &mut dyn Storage,
    round: &Round,
  ) -> Result<(), ContractError> {
    Ok(())
  }

  /// Increment royalty Claims for royalty recipients without autosend.
  fn upsert_royalty_claims(
    storage: &mut dyn Storage,
    config: &Config,
    round: &Round,
  ) -> Result<(), ContractError> {
    let total = Uint128::from(round.counts.tickets) * config.ticket_price;
    for royalty in config
      .royalties
      .iter()
      .filter(|x| !x.autosend.unwrap_or(false))
    {
      let amount_incr = Uint128::from(royalty.pct) * total / Uint128::from(100u128);
      CLAIMS.update(
        storage,
        royalty.address.clone(),
        |some_claim| -> Result<Claim, ContractError> {
          if let Some(mut claim) = some_claim {
            claim.amount += amount_incr;
            Ok(claim)
          } else {
            Ok(Claim {
              wallet: royalty.address.clone(),
              amount: amount_incr,
            })
          }
        },
      )?;
    }
    Ok(())
  }

  fn pick_winners_and_upsert_claims(
    storage: &mut dyn Storage,
    config: &Config,
    round: &Round,
    balance: Uint128,
    seed: &String,
  ) -> Result<(), ContractError> {
    let mut rng = pcg64_from_seed(&seed)?;

    let mut sample_pool: Vec<u32> = Vec::with_capacity(round.counts.tickets as usize);
    let mut all_wallets: Vec<Addr> = Vec::with_capacity(round.counts.wallets as usize);

    // create the sample pool to select winner address indices from
    for (i, result) in PLAYERS
      .prefix(round.index)
      .range(storage, None, None, Order::Ascending)
      .enumerate()
    {
      if let Some((addr, player)) = result.ok() {
        all_wallets.push(addr.clone());
        sample_pool.append(
          &mut iter::repeat(i as u32)
            .take(player.ticket_count as usize)
            .collect(),
        );
      }
    }
    let claim_pcts = Lottery::calculate_claim_percentages(config, round)?;
    let n_selections = claim_pcts.len() as u32;
    let mut visited: HashSet<Addr> = HashSet::with_capacity(n_selections as usize);
    let mut winner_index = 0u32;

    while winner_index < n_selections {
      let i = rng.next_u64() % sample_pool.len() as u64;
      let wallet = &all_wallets[i as usize];
      let previously_selected = visited.contains(wallet);

      // take the wallet if we're in a "multiwin" game, where there's selection
      // WITH replacement, or else take it if we haven't seen the wallet before.
      if config.selection.with_replacement || !previously_selected {
        let claim_amount = apply_pct(balance, claim_pcts[winner_index as usize]);
        visited.insert(wallet.clone());
        Self::upsert_claim(storage, wallet, claim_amount)?;
        winner_index += 1;
      }
    }

    Ok(())
  }

  fn upsert_claim(
    storage: &mut dyn Storage,
    wallet: &Addr,
    amount: Uint128,
  ) -> Result<(), ContractError> {
    CLAIMS.update(
      storage,
      wallet.clone(),
      |some_claim| -> Result<Claim, ContractError> {
        if let Some(mut claim) = some_claim {
          claim.amount += amount;
          Ok(claim)
        } else {
          Ok(Claim {
            wallet: wallet.clone(),
            amount,
          })
        }
      },
    )?;
    Ok(())
  }

  /// Based on the config params, return a vec containing a pct int (value between
  /// 0..100), specifying the "claim" percent owed to each winner's wallet
  /// according to their place -- e.g. 1st place, 2nd place.
  fn calculate_claim_percentages(
    config: &Config,
    round: &Round,
  ) -> StdResult<Vec<u8>> {
    match config.selection.method.clone() {
      WinnerSelectionMethod::Fixed(split) => {
        // a "fixed" winner count means that the % won by each winning wallet is
        // specified by a static vec of pcts, like [70, 30], for example.  if
        // there are fewer wallets available than the split vec, we return a slice
        // instead of the full vec.
        let n_winners = std::cmp::min(round.counts.wallets, split.len() as u32);
        Ok(Vec::from_slice(&split[..n_winners as usize])?)
      },
      WinnerSelectionMethod::Percent { pct, max } => {
        // a "percentage" based winner counts means that the number of drawings is
        // derived as a percent of the total number of ticket-holding wallets, or
        // at least 1. If a "max" is defined, then we cap the maximum number of
        // drawings.
        let mut n_winners = std::cmp::max(1, round.counts.wallets * (pct as u32) / 100);
        if let Some(n_max_winner) = max {
          n_winners = n_winners.max(n_max_winner);
        }
        // return a vec of identical percentages for each winner
        let pct = (100 / n_winners) as u8;
        Ok(iter::repeat(pct).take(n_winners as usize).collect())
      },
    }
  }
}
