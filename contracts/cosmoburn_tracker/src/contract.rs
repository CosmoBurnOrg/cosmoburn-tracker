use crate::functions::check_is_admin;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use astroport::asset::validate_native_denom;
use astroport::tokenfactory_tracker::SudoMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Storage, Uint128};
use cw2::set_contract_version;
use std::collections::HashMap;

use crate::error::ContractError;
use crate::state::{Config, State, BALANCES, CONFIG, STATE, TOTAL_SUPPLY_HISTORY};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    deps.api.addr_validate(&msg.tokenfactory_module_address)?;

    validate_native_denom(&msg.tracked_denom)?;

    let config = Config {
        d: msg.tracked_denom.clone(),
        m: msg.tokenfactory_module_address,
        admin_addr: deps.api.addr_validate(&msg.admin_addr)?.to_string(),
    };

    let state = State {
        excluded_wallets: HashMap::new(),
    };

    STATE.save(deps.storage, &state)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("contract", CONTRACT_NAME)
        .add_attribute("tracked_denom", config.d)
        .add_attribute("tokenfactory_module_address", config.m))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExcludeWallet { addr, memo } => try_exclude_wallet(deps, info, addr, memo),

        ExecuteMsg::IncludeWallet { addr } => try_include_wallet(deps, info, addr),
    }
}

// Excludes wallet address from receiving distributed tokens.
// addr: wallet address to be excluded.
pub fn try_exclude_wallet(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
    memo: String,
) -> Result<Response, ContractError> {
    check_is_admin(&deps, info)?;

    validate_native_denom(&addr)?;


    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if state.excluded_wallets.contains_key(&addr.to_string()) {
            return Err(ContractError::TokenAlreadyWhitelisted {});
        }

        state
            .excluded_wallets
            .insert(addr.to_string(), memo);
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_exclude_wallet")
        .add_attribute("addr", addr.clone()))
}

// Removes addres from exluded wallets.
// addr: wallet address to be removed from excluded wallets.
pub fn try_include_wallet(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    check_is_admin(&deps, info)?;

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if state.excluded_wallets.remove(&addr.clone()) == None {
            return Err(ContractError::TokenNotFound {});
        };
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_include_wallet")
        .add_attribute("addr", addr.clone()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        
        SudoMsg::BlockBeforeSend { from, to, amount } => {
            let config = CONFIG.load(deps.storage)?;

            if amount.denom != config.d {
                Err(ContractError::InvalidDenom {
                    expected_denom: config.d,
                })
            } else {

                track_balances(
                    deps.storage,
                    env.block.time.seconds(),
                    &config,
                    from,
                    to,
                    amount.amount,
                )
            }
        }

        SudoMsg::TrackBeforeSend { .. } => Ok(Response::default()),
    }
}


pub fn track_balances(
    storage: &mut dyn Storage,
    block_seconds: u64,
    config: &Config,
    from: String,
    to: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // If the token is minted directly to an address, we don't need to subtract
    // as the sender is the module address
    let state = STATE.load(storage)?;

    let is_excluded_from = state.excluded_wallets.contains_key(&from);
    let is_excluded_to = state.excluded_wallets.contains_key(&to);

    if from.ne(&config.m) && !is_excluded_from {
        BALANCES.update::<_, StdError>(storage, &from, block_seconds, |balance| {
            balance
                .unwrap_or_default()
                .checked_sub(amount)
                .map_err(|err| {
                    StdError::generic_err(format!(
                        "{err}: send from {from} to {to} amount {amount} block_seconds {block_seconds}"
                    ))
                })
        })?;
    } else {
        // Minted new tokens or excluded wallet
        TOTAL_SUPPLY_HISTORY.update::<_, StdError>(storage, block_seconds, |balance| {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        })?;
    }

    // When burning tokens, the receiver is the token factory module address
    // Sending tokens to the module address isn't allowed by the chain
    if to.ne(&config.m) && !is_excluded_to {
        BALANCES.update::<_, StdError>(storage, &to, block_seconds, |balance| {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        })?;
    } else {
        // Burned tokens or sent to excluded wallet
        TOTAL_SUPPLY_HISTORY.update::<_, StdError>(storage, block_seconds, |balance| {
            balance
                .unwrap_or_default()
                .checked_sub(amount)
                .map_err(|err| {
                    StdError::generic_err(format!(
                        "{err}: from {from} to {to} amount {amount} block_seconds {block_seconds}"
                    ))
                })
        })?;
    }

    Ok(Response::default())
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies_with_balance, mock_env, mock_info, MockApi, MockQuerier, MockStorage
    };
    use cosmwasm_std::{coins};
    use cosmwasm_std::OwnedDeps;
    use cosmwasm_std::{Addr, Uint128};

    const USER: &str = "neutron1";
    const ADMIN: &str = "neutron2";
    const TOKEN_FACTORY: &str = "neutron4";
    const NATIVE_DENOM: &str = "untrn";

    fn proper_initialization() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies_with_balance(&coins(2, NATIVE_DENOM));

        let msg = InstantiateMsg {
            admin_addr: Addr::unchecked(ADMIN).to_string(),
            tokenfactory_module_address: Addr::unchecked(TOKEN_FACTORY).to_string(),
            tracked_denom: NATIVE_DENOM.to_string(),
        };

        let info = mock_info(ADMIN, &coins(1000, NATIVE_DENOM.to_string()));

        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        deps
    }

    #[test]
    fn track_mint() {
        let mut deps = proper_initialization();
        let _info = mock_info(ADMIN, &coins(1000, NATIVE_DENOM.to_string()));

        let config = Config {
            d: NATIVE_DENOM.to_string(),
            m: TOKEN_FACTORY.to_string(),
            admin_addr: ADMIN.to_string(),
        };
        
        let _res:Result<Response, ContractError> = track_balances(
            &mut deps.storage,
            mock_env().block.time.seconds(),
            &config,
            TOKEN_FACTORY.to_string(),
            USER.to_string(),
            Uint128::new(999),
        );
        
        assert_eq!(TOTAL_SUPPLY_HISTORY.load(&mut deps.storage).unwrap(), Uint128::new(999));
    }

    #[test]
    fn track_burn() {
        let mut deps = proper_initialization();
        let _info = mock_info(ADMIN, &coins(1000, NATIVE_DENOM.to_string()));

        let config = Config {
            d: NATIVE_DENOM.to_string(),
            m: TOKEN_FACTORY.to_string(),
            admin_addr: ADMIN.to_string(),
        };
        
        let _res:Result<Response, ContractError> = track_balances(
            &mut deps.storage,
            mock_env().block.time.seconds(),
            &config,
            TOKEN_FACTORY.to_string(),
            USER.to_string(),
            Uint128::new(100),
        );
        
        let _res:Result<Response, ContractError> = track_balances(
            &mut deps.storage,
            mock_env().block.time.seconds(),
            &config,
            USER.to_string(),
            TOKEN_FACTORY.to_string(),
            Uint128::new(10),
        );
        
        assert_eq!(TOTAL_SUPPLY_HISTORY.load(&mut deps.storage).unwrap(), Uint128::new(90));
    }

    #[test]
    fn track_balance() {
        let mut deps = proper_initialization();
        let _info = mock_info(ADMIN, &coins(1000, NATIVE_DENOM.to_string()));

        let config = Config {
            d: NATIVE_DENOM.to_string(),
            m: TOKEN_FACTORY.to_string(),
            admin_addr: ADMIN.to_string(),
        };
        
        let _res:Result<Response, ContractError> = track_balances(
            &mut deps.storage,
            mock_env().block.time.seconds(),
            &config,
            TOKEN_FACTORY.to_string(),
            USER.to_string(),
            Uint128::new(100),
        );
        
        assert_eq!(BALANCES.may_load(&deps.storage, &USER).unwrap(), Some(Uint128::new(100)));
    }

}