use crate::datatype::{Contributor, DataKey, StabilizationError, StabilizationFund};
use crate::interface::FundManagement;
use crate::PriceStabilizationContractArgs;
use crate::{PriceStabilizationContract, PriceStabilizationContractClient};
use soroban_sdk::{contractimpl, Address, Bytes, BytesN, Env, Map, String};

#[contractimpl]
impl FundManagement for PriceStabilizationContract {
    fn create_fund(
        env: Env,
        admin: Address,
        fund_name: String,
        price_threshold: i128,
        crop_type: String,
    ) -> Result<BytesN<32>, StabilizationError> {
        // Verify admin authorization
        admin.require_auth();

        // Verify admin is the contract admin
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(StabilizationError::Unauthorized);
        }

        // Validate inputs
        if price_threshold <= 0 || fund_name.is_empty() || crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }

        // Generate a unique fund ID using the env.crypto API
        // Generate a unique ID using timestamp and additional entropy
        let timestamp = env.ledger().timestamp();
        let mut bytes_data = Bytes::new(&env);

        // Add timestamp bytes
        for b in timestamp.to_be_bytes().iter() {
            bytes_data.push_back(*b);
        }

        // Add sequence number for additional entropy
        let sequence = env.ledger().sequence();
        for b in sequence.to_be_bytes().iter() {
            bytes_data.push_back(*b);
        }

        // Create a unique fund ID
        let fund_id = env.crypto().sha256(&bytes_data);

        // Check if fund with this ID already exists (extremely unlikely but good practice)
        if env
            .storage()
            .persistent()
            .has(&DataKey::Fund(fund_id.clone().into()))
        {
            return Err(StabilizationError::FundAlreadyExists);
        }

        // Create the fund
        let fund = StabilizationFund {
            fund_id: fund_id.clone().into(),
            fund_name,
            admin: admin.clone(),
            total_balance: 0,
            price_threshold,
            crop_type,
            active: true,
            creation_time: env.ledger().timestamp(),
            last_payout_time: None,
        };

        // Store the fund
        env.storage()
            .persistent()
            .set(&DataKey::Fund(fund_id.clone().into()), &fund);

        Ok(fund_id.into())
    }

    fn contribute_fund(
        env: Env,
        contributor: Address,
        fund_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), StabilizationError> {
        // Verify contributor authorization
        contributor.require_auth();

        // Validate inputs
        if amount <= 0 {
            return Err(StabilizationError::InvalidInput);
        }

        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }

        // Get the fund
        let mut fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();

        // Check if fund is active
        if !fund.active {
            return Err(StabilizationError::FundNotFound);
        }

        // Update fund balance
        fund.total_balance += amount;

        // Update or create contributor record
        let contributor_key = DataKey::Contributor(fund_id.clone(), contributor.clone());
        let timestamp = env.ledger().timestamp();

        let contributor_record = if env.storage().persistent().has(&contributor_key) {
            let mut record: Contributor = env.storage().persistent().get(&contributor_key).unwrap();
            record.total_contribution += amount;
            record.last_contribution_time = timestamp;
            record
        } else {
            Contributor {
                address: contributor.clone(),
                total_contribution: amount,
                last_contribution_time: timestamp,
            }
        };

        // Store updated records
        env.storage().persistent().set(&fund_key, &fund);
        env.storage()
            .persistent()
            .set(&contributor_key, &contributor_record);

        Ok(())
    }

    fn get_fund_status(
        env: Env,
        fund_id: BytesN<32>,
    ) -> Result<Map<String, i128>, StabilizationError> {
        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }

        // Get the fund
        let fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();

        // Create a map to return fund status
        let mut status = Map::new(&env);
        status.set(String::from_str(&env, "total_balance"), fund.total_balance);
        status.set(
            String::from_str(&env, "price_threshold"),
            fund.price_threshold,
        );

        // Add timestamp information
        status.set(
            String::from_str(&env, "creation_time"),
            fund.creation_time as i128,
        );
        if let Some(last_payout) = fund.last_payout_time {
            status.set(
                String::from_str(&env, "last_payout_time"),
                last_payout as i128,
            );
        }

        Ok(status)
    }

    fn update_price_threshold(
        env: Env,
        admin: Address,
        fund_id: BytesN<32>,
        new_threshold: i128,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();

        // Validate inputs
        if new_threshold <= 0 {
            return Err(StabilizationError::InvalidInput);
        }

        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }

        // Get the fund
        let mut fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();

        // Verify admin is the fund admin
        if admin != fund.admin {
            return Err(StabilizationError::Unauthorized);
        }

        // Update the threshold
        fund.price_threshold = new_threshold;

        // Store updated fund
        env.storage().persistent().set(&fund_key, &fund);

        Ok(())
    }
}
