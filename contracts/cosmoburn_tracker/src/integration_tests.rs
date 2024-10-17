#[cfg(test)]
mod tests {
    
    


    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    


    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::query::query,
        );
        Box::new(contract)
    }

    const USER: &str = "neutron1";
    const ADMIN: &str = "neutron2";
    const TOKEN_FACTORY: &str = "neutron4";
    const NATIVE_DENOM: &str = "untrn";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            admin_addr: Addr::unchecked(ADMIN).to_string(),
            tokenfactory_module_address: Addr::unchecked(TOKEN_FACTORY).to_string(),
            tracked_denom: NATIVE_DENOM.to_string(),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod track {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg, ExcludedWalletsResponse, ListHoldersResponse};

        #[test]
        fn exclude_wallet() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg::ExcludeWallet {
                addr: "untrn".to_string(),
                memo: "native1".to_string(),
            };

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            let query_res: ExcludedWalletsResponse = app
            .wrap()
            .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetExcludedWallets {})
            .unwrap();

            assert_eq!(query_res.excludedwallets.len(), 1);

            let msg = ExecuteMsg::IncludeWallet {
                addr: "untrn".to_string(),
            };

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            let query_res: ExcludedWalletsResponse = app
            .wrap()
            .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetExcludedWallets {})
            .unwrap();

            assert_eq!(query_res.excludedwallets.len(), 0);
        }

        #[test]
        fn queries() {
            let (app, cw_template_contract) = proper_instantiate();

            let query_res: Uint128 = app
            .wrap()
            .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::BalanceAt { address: ADMIN.to_string(), timestamp: Some(1u64) })
            .unwrap();

            assert_eq!(query_res, Uint128::new(0));

            let query_res: Uint128 = app
            .wrap()
            .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::TotalSupplyAt { timestamp: Some(1u64) })
            .unwrap();

            assert_eq!(query_res, Uint128::new(0));

            let query_res: ListHoldersResponse = app
            .wrap()
            .query_wasm_smart(&cw_template_contract.addr(), &QueryMsg::GetHolders { from: None, limit: Some(10)})
            .unwrap();

            assert_eq!(query_res.holders.len(), 0);
            
        }

    }
}
