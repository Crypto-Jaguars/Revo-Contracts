use crate::datatype::{
    DataKey, Farmer, FarmerCrop, Payout, PriceData, StabilizationError, StabilizationFund,
};
use crate::interface::DistributionManagement;
use crate::PriceStabilizationContractArgs;
use crate::{PriceStabilizationContract, PriceStabilizationContractClient};
use soroban_sdk::{contractimpl, Address, BytesN, Env, Map, String, Vec};

#[contractimpl]
impl DistributionManagement for PriceStabilizationContract {
    fn trigger_payout(
        env: Env,
        admin: Address,
        fund_id: BytesN<32>,
        farmers: Vec<Address>,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();

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

        // Check if there are farmers to distribute to
        if farmers.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }

        // Get the current market price
        let price_key = DataKey::MarketPrice(fund.crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }

        let price_data: PriceData = env.storage().persistent().get(&price_key).unwrap();

        // Check if price is below threshold
        if price_data.price >= fund.price_threshold {
            return Err(StabilizationError::ThresholdNotReached);
        }

        // Calculate the price difference with overflow protection
        let price_difference = fund
            .price_threshold
            .checked_sub(price_data.price)
            .ok_or(StabilizationError::InvalidInput)?;

        // Collect valid farmers and validate their eligibility
        let mut valid_farmers = Vec::new(&env);

        for farmer_address in farmers.iter() {
            // Check if farmer is registered
            let farmer_key = DataKey::Farmer(farmer_address.clone());
            if !env.storage().persistent().has(&farmer_key) {
                continue; // Skip unregistered farmers
            }

            let farmer: Farmer = env.storage().persistent().get(&farmer_key).unwrap();

            // Check if farmer is active
            if !farmer.active {
                continue;
            }

            // Check if farmer grows this crop type
            let farmer_crop_key =
                DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            if !env.storage().persistent().has(&farmer_crop_key) {
                continue;
            }

            valid_farmers.push_back(farmer_address.clone());
        }

        // Check if there are valid farmers
        if valid_farmers.is_empty() {
            return Err(StabilizationError::FarmerNotRegistered);
        }

        // Calculate actual total payout needed
        let mut total_payout_needed: i128 = 0;
        for farmer_address in valid_farmers.iter() {
            let farmer_crop_key =
                DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            let farmer_crop: FarmerCrop = env.storage().persistent().get(&farmer_crop_key).unwrap();
            let farmer_payout = price_difference
                .checked_mul(farmer_crop.production_capacity)
                .ok_or(StabilizationError::InvalidInput)?;
            total_payout_needed = total_payout_needed
                .checked_add(farmer_payout)
                .ok_or(StabilizationError::InvalidInput)?;
        }

        if fund.total_balance < total_payout_needed {
            return Err(StabilizationError::InsufficientFunds);
        }

        let timestamp = env.ledger().timestamp();

        // Distribute payouts to farmers
        for farmer_address in valid_farmers.iter() {
            let farmer_crop_key =
                DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            let farmer_crop: FarmerCrop = env.storage().persistent().get(&farmer_crop_key).unwrap();

            let payout_amount = price_difference
                .checked_mul(farmer_crop.production_capacity)
                .ok_or(StabilizationError::InvalidInput)?;

            // Create payout record
            let payout = Payout {
                farmer_id: farmer_address.clone(),
                fund_id: fund_id.clone(),
                amount: payout_amount,
                timestamp,
                market_price: price_data.price,
                threshold_price: fund.price_threshold,
            };

            // Store payout record
            let payout_key = DataKey::Payout(fund_id.clone(), farmer_address.clone(), timestamp);
            env.storage().persistent().set(&payout_key, &payout);

            // Update farmer's total received payouts
            let farmer_key = DataKey::Farmer(farmer_address.clone());
            let mut farmer: Farmer = env.storage().persistent().get(&farmer_key).unwrap();
            farmer.total_received_payouts = farmer
                .total_received_payouts
                .checked_add(payout_amount)
                .ok_or(StabilizationError::InvalidInput)?;
            env.storage().persistent().set(&farmer_key, &farmer);
        }

        // Update fund balance
        fund.total_balance = fund
            .total_balance
            .checked_sub(total_payout_needed)
            .ok_or(StabilizationError::InvalidInput)?;
        fund.last_payout_time = Some(timestamp);
        env.storage().persistent().set(&fund_key, &fund);

        Ok(())
    }

    fn register_farmer(
        env: Env,
        admin: Address,
        farmer: Address,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();

        // Validate inputs
        if !env.storage().persistent().has(&DataKey::Admin) {
            return Err(StabilizationError::Unauthorized);
        }

        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(StabilizationError::Unauthorized);
        }

        // Check if farmer is already registered
        let farmer_key = DataKey::Farmer(farmer.clone());
        if env.storage().persistent().has(&farmer_key) {
            return Err(StabilizationError::FarmerAlreadyRegistered);
        }

        // Register farmer
        let farmer_data = Farmer {
            address: farmer.clone(),
            registered_time: env.ledger().timestamp(),
            total_received_payouts: 0,
            active: true,
        };

        env.storage().persistent().set(&farmer_key, &farmer_data);

        Ok(())
    }

    fn register_farmer_crop(
        env: Env,
        admin: Address,
        farmer: Address,
        crop_type: String,
        production_capacity: i128,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();

        // Validate inputs
        if crop_type.is_empty() || production_capacity <= 0 {
            return Err(StabilizationError::InvalidInput);
        }

        // Check if farmer is registered
        let farmer_key = DataKey::Farmer(farmer.clone());
        if !env.storage().persistent().has(&farmer_key) {
            return Err(StabilizationError::FarmerNotRegistered);
        }

        // Register farmer's crop
        let farmer_crop_key = DataKey::FarmerCrops(farmer.clone(), crop_type.clone());

        // Check if farmer already registered this crop
        if env.storage().persistent().has(&farmer_crop_key) {
            return Err(StabilizationError::CropAlreadyRegistered);
        }

        let farmer_crop = FarmerCrop {
            crop_type: crop_type.clone(),
            production_capacity,
        };

        env.storage()
            .persistent()
            .set(&farmer_crop_key, &farmer_crop);

        Ok(())
    }

    fn get_farmer_payouts(
        env: Env,
        fund_id: BytesN<32>,
        farmer: Address,
    ) -> Result<Vec<Map<String, i128>>, StabilizationError> {
        // Check if farmer is registered
        let farmer_key = DataKey::Farmer(farmer.clone());
        if !env.storage().persistent().has(&farmer_key) {
            return Err(StabilizationError::FarmerNotRegistered);
        }

        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }

        // Collect payout history
        let history = Vec::new(&env);

        // This is a simplified implementation - in a real scenario,
        // you would need to iterate through all payout records
        // For now, we'll return an empty vector as a placeholder

        Ok(history)
    }
}
