#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult, Uint128};

use cw_paginate::paginate_snapshot_map_keys;
use crate::state::{BALANCES, CONFIG, TOTAL_SUPPLY_HISTORY, STATE};
use crate::msg::{HolderBalanceResponse, ListHoldersResponse, QueryMsg, ConfigResponse, ExcludedWalletsResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BalanceAt { address, timestamp } => {
            to_json_binary(&balance_at(deps, env, address, timestamp)?)
        }
        QueryMsg::TotalSupplyAt { timestamp } => {
            to_json_binary(&total_supply_at(deps, env, timestamp)?)
        }
        QueryMsg::GetHolders { from, limit, timestamp } => {
            to_json_binary(&query_list_holders(deps, env, from, limit, timestamp)?)
        }
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&ConfigResponse {
                tracked_denom: config.d,
                token_factory_module: config.m,
            })
        }
        QueryMsg::GetExcludedWallets {} => {
            to_json_binary(&query_excludedwallets(deps)?)
        }
    }
}

pub fn query_list_holders(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    timestamp: Option<u64>,
) -> StdResult<ListHoldersResponse> {

    let holders = paginate_snapshot_map_keys(
        deps,
        &BALANCES,
        start_after.as_deref(),
        limit.or(Some(10u32)),
        cosmwasm_std::Order::Ascending,
    )?;

    let holders = holders
        .into_iter()
        .map(|address| HolderBalanceResponse {
            address: address.clone(),
            balance: balance_at(deps, env.clone(), address, timestamp).unwrap_or(Uint128::zero()).into(),
        })
        .collect();

    Ok(ListHoldersResponse { holders })
}

fn balance_at(deps: Deps, env: Env, address: String, timestamp: Option<u64>) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    match timestamp.unwrap_or(block_time) {
        timestamp if timestamp == block_time => BALANCES.may_load(deps.storage, &address),
        timestamp => BALANCES.may_load_at_height(deps.storage, &address, timestamp),
    }
    .map(|balance| balance.unwrap_or_default())
}

fn total_supply_at(deps: Deps, env: Env, timestamp: Option<u64>) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    match timestamp.unwrap_or(block_time) {
        timestamp if timestamp == block_time => TOTAL_SUPPLY_HISTORY.may_load(deps.storage),
        timestamp => TOTAL_SUPPLY_HISTORY.may_load_at_height(deps.storage, timestamp),
    }
    .map(|total_supply| total_supply.unwrap_or_default())
}

fn query_excludedwallets(deps: Deps) -> StdResult<ExcludedWalletsResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(ExcludedWalletsResponse {
        excludedwallets: state.excluded_wallets.clone(),
    })
}