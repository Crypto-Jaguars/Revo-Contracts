#![cfg(test)]
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, BytesN, Env, Map, String};

use crate::{storage, validate, CommodityTokenContract, Inventory};

pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
}

impl TestContext {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
    let contract_id: Address = env.register(CommodityTokenContract, ());
        let admin = Address::generate(&env);
        Self { env, contract_id, admin }
    }

    pub fn set_time(&self, ts: u64) { self.env.ledger().set_timestamp(ts); }

    pub fn now(&self) -> u64 {
        self.env.ledger().timestamp()
    }

    pub fn init_with_admin(&self) {
        self.env.as_contract(&self.contract_id, || {
            let _ = CommodityTokenContract::initialize(self.env.clone(), self.admin.clone());
        });
    }

    pub fn add_inventory(&self, commodity_type: &str, qty: u32) {
        self.env.as_contract(&self.contract_id, || {
            // Manually set admin and update inventory to avoid double require_auth in wrapper
            storage::set_admin(&self.env, &self.admin);
            let ct = String::from_str(&self.env, commodity_type);
            let mut inv = storage::get_inventory(&self.env, &ct);
            inv.total_quantity = inv.total_quantity.saturating_add(qty);
            inv.available_quantity = inv.available_quantity.saturating_add(qty);
            storage::update_inventory(&self.env, &ct, &inv).unwrap();
        });
    }

    pub fn register_verification(&self, commodity_type: &str, vbytes: [u8; 32]) -> BytesN<32> {
        let verification = BytesN::from_array(&self.env, &vbytes);
        self.env.as_contract(&self.contract_id, || {
            let ct = String::from_str(&self.env, commodity_type);
            let mut meta: Map<String, String> = Map::new(&self.env);
            meta.set(String::from_str(&self.env, "issuer"), String::from_str(&self.env, "lab-a"));
            // set admin to satisfy auth check
            storage::set_admin(&self.env, &self.admin);
            let _ = validate::register_commodity_verification(
                &self.env,
                &self.admin,
                &ct,
                &verification,
                &meta,
            );
        });
        verification
    }

    pub fn issue_token(&self,
        issuer: &Address,
        commodity_type: &str,
        qty: u32,
        grade: &str,
        storage_location: &str,
        expires_in_secs: u64,
        verification: &BytesN<32>,
    ) -> BytesN<32> {
        let now = self.env.ledger().timestamp();
        let token_id = self.env.as_contract(&self.contract_id, || {
            let ct = String::from_str(&self.env, commodity_type);
            let g = String::from_str(&self.env, grade);
            let sl = String::from_str(&self.env, storage_location);
            CommodityTokenContract::issue_token(
                self.env.clone(),
                issuer.clone(),
                ct,
                qty,
                g,
                sl,
                now + expires_in_secs,
                verification.clone(),
            ).expect("issue ok")
        });
        token_id
    }

    pub fn get_inventory(&self, commodity_type: &str) -> Inventory {
        self.env.as_contract(&self.contract_id, || {
            CommodityTokenContract::list_available_inventory(
                self.env.clone(),
                String::from_str(&self.env, commodity_type),
            )
        })
    }
}
