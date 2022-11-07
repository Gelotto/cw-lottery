use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("ValidationError")]
  ValidationError { reason: Option<String> },

  #[error("InactiveRound")]
  InactiveRound {},

  #[error("TooManyTickets")]
  TooManyTickets { max_tickets_per_wallet: u32 },

  #[error("Forbidden")]
  Forbidden {},

  #[error("NotActive")]
  NotActive {},

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("FundsInvalid")]
  FundsInvalid { reason: String },

  #[error("RoundNotFound")]
  RoundNotFound {},

  #[error("InsufficientFunds")]
  InsufficientFunds {},

  #[error("ExcessiveFunds")]
  ExcessiveFunds {},

  #[error("NotCanceled")]
  NotCanceled {},

  #[error("PlayerNotFound")]
  PlayerNotFound {},

  #[error("MissingRewards")]
  MissingRewards {},

  #[error("InvalidSeed")]
  InvalidSeed {},
}
