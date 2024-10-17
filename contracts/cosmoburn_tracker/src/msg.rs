use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::QueryResponse;
use cosmwasm_std::Uint128;
use cosmwasm_schema::cw_serde;
use std::collections::HashMap;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub tracked_denom: String,
    pub tokenfactory_module_address: String,
    pub admin_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ExcludeWallet {
        addr: String,
        memo: String,
    },

    IncludeWallet {
        addr: String,
    },
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HolderBalanceResponse {
    pub address: String,
    pub balance: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListHoldersResponse { 
    pub holders: Vec<HolderBalanceResponse>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return the balance of the given address at the given timestamp.
    #[returns(Uint128)]
    BalanceAt {
        address: String,
        timestamp: Option<u64>,
    },
    /// Return the total supply at the given timestamp.
    #[returns(Uint128)]
    TotalSupplyAt { timestamp: Option<u64> },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ListHoldersResponse)]
    GetHolders {
        from: Option<String>,
        limit: Option<u32>,
        timestamp: Option<u64>,
    },
    #[returns(QueryResponse)]
    GetExcludedWallets {},
    
}

#[cw_serde]
pub struct ConfigResponse {
    /// Tracked denom
    pub tracked_denom: String,
    /// Token factory module address
    pub token_factory_module: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExcludedWalletsResponse {
    pub excludedwallets: HashMap<String, String>,
}