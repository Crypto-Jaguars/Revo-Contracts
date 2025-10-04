use crate::{FarmerLiquidityPoolContract, FarmerLiquidityPoolContractClient};
use soroban_sdk::{
    testutils::Address as _, token, token::StellarAssetClient, Address, Env, IntoVal,
};

pub fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, token::Client<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    let address = contract_address.address();
    let client = token::Client::new(env, &address);
    (address, client)
}

pub fn create_token_contract_with_initial_supply<'a>(
    env: &Env,
    admin: &Address,
    initial_supply: i128,
) -> (Address, token::Client<'a>) {
    let (contract_address, client) = create_token_contract(env, admin);

    // Use StellarAssetClient to mint initial supply
    let stellar_client = StellarAssetClient::new(env, &contract_address);

    // Set up authentication context for minting
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_address,
            fn_name: "mint",
            args: (admin.clone(), initial_supply).into_val(env),
            sub_invokes: &[],
        },
    }]);

    stellar_client.mint(admin, &initial_supply);

    (contract_address, client)
}

pub fn create_pool_contract(env: &Env) -> FarmerLiquidityPoolContractClient {
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    FarmerLiquidityPoolContractClient::new(env, &contract_id)
}

pub fn setup_test_environment(env: &Env) -> TestEnvironment {
    let admin = Address::generate(env);
    let user1 = Address::generate(env);
    let user2 = Address::generate(env);
    let user3 = Address::generate(env);

    // Create token contracts with initial supply
    let (token_a, token_a_client) =
        create_token_contract_with_initial_supply(env, &admin, 1_000_000);
    let (token_b, token_b_client) =
        create_token_contract_with_initial_supply(env, &admin, 1_000_000);

    // Distribute tokens to users with proper authentication
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_a,
            fn_name: "transfer",
            args: (admin.clone(), user1.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_a_client.transfer(&admin, &user1, &100_000);

    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_a,
            fn_name: "transfer",
            args: (admin.clone(), user2.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_a_client.transfer(&admin, &user2, &100_000);

    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_a,
            fn_name: "transfer",
            args: (admin.clone(), user3.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_a_client.transfer(&admin, &user3, &100_000);

    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_b,
            fn_name: "transfer",
            args: (admin.clone(), user1.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_b_client.transfer(&admin, &user1, &100_000);

    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_b,
            fn_name: "transfer",
            args: (admin.clone(), user2.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_b_client.transfer(&admin, &user2, &100_000);

    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &token_b,
            fn_name: "transfer",
            args: (admin.clone(), user3.clone(), 100_000i128).into_val(env),
            sub_invokes: &[],
        },
    }]);
    token_b_client.transfer(&admin, &user3, &100_000);

    let pool_contract = create_pool_contract(env);

    TestEnvironment {
        env: env.clone(),
        admin,
        user1,
        user2,
        user3,
        token_a,
        token_b,
        token_a_client,
        token_b_client,
        pool_contract,
    }
}

pub struct TestEnvironment<'a> {
    pub env: Env,
    pub admin: Address,
    pub user1: Address,
    pub user2: Address,
    pub user3: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub token_a_client: token::Client<'a>,
    pub token_b_client: token::Client<'a>,
    pub pool_contract: FarmerLiquidityPoolContractClient<'a>,
}

impl<'a> TestEnvironment<'a> {
    pub fn initialize_pool(&self, fee_rate: u32) {
        self.pool_contract
            .initialize(&self.admin, &self.token_a, &self.token_b, &fee_rate);
    }

    pub fn add_liquidity(&self, provider: &Address, amount_a: i128, amount_b: i128) -> i128 {
        // Approve tokens first with proper authentication
        self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: provider,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &self.token_a,
                fn_name: "approve",
                args: (
                    provider.clone(),
                    self.pool_contract.address.clone(),
                    amount_a,
                    1000u32,
                )
                    .into_val(&self.env),
                sub_invokes: &[],
            },
        }]);
        self.token_a_client
            .approve(provider, &self.pool_contract.address, &amount_a, &1000);

        self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: provider,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &self.token_b,
                fn_name: "approve",
                args: (
                    provider.clone(),
                    self.pool_contract.address.clone(),
                    amount_b,
                    1000u32,
                )
                    .into_val(&self.env),
                sub_invokes: &[],
            },
        }]);
        self.token_b_client
            .approve(provider, &self.pool_contract.address, &amount_b, &1000);

        // Set up authentication for the contract's transfer calls
        self.env.mock_auths(&[
            soroban_sdk::testutils::MockAuth {
                address: provider,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_a,
                    fn_name: "transfer",
                    args: (
                        provider.clone(),
                        self.pool_contract.address.clone(),
                        amount_a,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            },
            soroban_sdk::testutils::MockAuth {
                address: provider,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_b,
                    fn_name: "transfer",
                    args: (
                        provider.clone(),
                        self.pool_contract.address.clone(),
                        amount_b,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            },
        ]);

        self.pool_contract
            .add_liquidity(provider, &amount_a, &amount_b, &0)
    }

    pub fn remove_liquidity(&self, provider: &Address, lp_tokens: i128) -> (i128, i128) {
        self.pool_contract
            .remove_liquidity(provider, &lp_tokens, &0, &0)
    }

    pub fn swap(&self, trader: &Address, token_in: &Address, amount_in: i128) -> i128 {
        // Approve token first with proper authentication
        if *token_in == self.token_a {
            self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: trader,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_a,
                    fn_name: "approve",
                    args: (
                        trader.clone(),
                        self.pool_contract.address.clone(),
                        amount_in,
                        1000u32,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            }]);
            self.token_a_client
                .approve(trader, &self.pool_contract.address, &amount_in, &1000);

            // Set up authentication for the contract's transfer call
            self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: trader,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_a,
                    fn_name: "transfer",
                    args: (
                        trader.clone(),
                        self.pool_contract.address.clone(),
                        amount_in,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            }]);
        } else {
            self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: trader,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_b,
                    fn_name: "approve",
                    args: (
                        trader.clone(),
                        self.pool_contract.address.clone(),
                        amount_in,
                        1000u32,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            }]);
            self.token_b_client
                .approve(trader, &self.pool_contract.address, &amount_in, &1000);

            // Set up authentication for the contract's transfer call
            self.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: trader,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &self.token_b,
                    fn_name: "transfer",
                    args: (
                        trader.clone(),
                        self.pool_contract.address.clone(),
                        amount_in,
                    )
                        .into_val(&self.env),
                    sub_invokes: &[],
                },
            }]);
        }

        self.pool_contract.swap(trader, token_in, &amount_in, &0)
    }

    pub fn get_pool_info(&self) -> crate::PoolInfo {
        self.pool_contract.get_pool_info()
    }

    pub fn get_reserves(&self) -> (i128, i128) {
        self.pool_contract.get_reserves()
    }

    pub fn get_lp_balance(&self, provider: &Address) -> i128 {
        self.pool_contract.get_lp_balance(provider)
    }

    pub fn calculate_swap_output(&self, token_in: &Address, amount_in: i128) -> i128 {
        self.pool_contract
            .calculate_swap_output(token_in, &amount_in)
    }

    pub fn claim_fees(&self, provider: &Address) -> (i128, i128) {
        self.pool_contract.claim_fees(provider)
    }

    pub fn get_accumulated_fees(&self, provider: &Address) -> (i128, i128) {
        self.pool_contract.get_accumulated_fees(provider)
    }
}

pub fn assert_approx_eq(actual: i128, expected: i128, tolerance: i128) {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };
    assert!(
        diff <= tolerance,
        "Expected {} but got {} (diff: {})",
        expected,
        actual,
        diff
    );
}

pub fn assert_balance(env: &Env, token: &Address, user: &Address, expected: i128) {
    let client = token::Client::new(env, token);
    let actual = client.balance(user);
    assert_eq!(
        actual, expected,
        "Token balance mismatch for user {:?}",
        user
    );
}

pub fn assert_pool_reserves<'a>(env: &TestEnvironment<'a>, expected_a: i128, expected_b: i128) {
    let (actual_a, actual_b) = env.get_reserves();
    assert_eq!(actual_a, expected_a, "Reserve A mismatch");
    assert_eq!(actual_b, expected_b, "Reserve B mismatch");
}

pub fn assert_lp_balance<'a>(env: &TestEnvironment<'a>, provider: &Address, expected: i128) {
    let actual = env.get_lp_balance(provider);
    assert_eq!(
        actual, expected,
        "LP balance mismatch for provider {:?}",
        provider
    );
}
