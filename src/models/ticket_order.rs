use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketOrder {
  pub wallet: Addr,
  pub ticket_count: u32,
  pub message: Option<String>,
  pub is_public: bool,
}
