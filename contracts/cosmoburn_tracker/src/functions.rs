use cosmwasm_std::{MessageInfo, DepsMut};
use crate::error::ContractError;
use crate::state::{CONFIG};



pub fn check_is_admin(deps: &DepsMut, info: MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    } else {
        return Ok(());
    }
}
