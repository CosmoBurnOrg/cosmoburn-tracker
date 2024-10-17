
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, SnapshotItem, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub excluded_wallets : HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub d: String,
    pub m: String,
    pub admin_addr: String,
}


pub const CONFIG: Item<Config> = Item::new("c");

pub const STATE: Item<State> = Item::new("state");


/// Contains snapshotted balances at every block.
pub const BALANCES: SnapshotMap<&str, Uint128> =
    SnapshotMap::new("b", "b_chpts", "b_chlg", Strategy::EveryBlock);

/// Contains the history of the total supply of the tracked denom
pub const TOTAL_SUPPLY_HISTORY: SnapshotItem<Uint128> =
    SnapshotItem::new("t", "t_chpts", "t_chlg", Strategy::EveryBlock);
