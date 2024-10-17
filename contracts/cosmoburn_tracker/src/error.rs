use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid denom, expected {expected_denom}")]
    InvalidDenom { expected_denom: String },

    #[error("Token already whitelisted")]
    TokenAlreadyWhitelisted {},

    #[error("Token not found")]
    TokenNotFound {},
}
