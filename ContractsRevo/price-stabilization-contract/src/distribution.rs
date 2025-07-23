use crate::datatype::{DataKey, Farmer, FarmerCrop, Payout, PriceData, StabilizationError, StabilizationFund};
use crate::PriceStabilizationContractArgs;
use crate::interface::PayoutDistribution;
use crate::{PriceStabilizationContract, PriceStabilizationContractClient};
use soroban_sdk::{contractimpl, Address, BytesN, Env, Map, String, Vec};

#[contractimpl]
impl PayoutDistribution for PriceStabilizationContract {
    fn trigger_payout(
        env: Env,
        admin: Address,
        fund_id: BytesN<32>,
        farmers: Vec<Address>,
    ) -> Result<Map<Address, i128>, StabilizationError> {
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
        
        // Calculate the price difference
        let price_difference = fund.price_threshold - price_data.price;
        
        // Calculate total production capacity of all farmers
        let mut total_capacity: i128 = 0;
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
            let farmer_crop_key = DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            if !env.storage().persistent().has(&farmer_crop_key) {
                continue;
            }
            
            let farmer_crop: FarmerCrop = env.storage().persistent().get(&farmer_crop_key).unwrap();
            total_capacity = total_capacity
                .checked_add(farmer_crop.production_capacity)
                .ok_or(StabilizationError::InvalidInput)?;
            valid_farmers.push_back(farmer_address.clone());
        }
        
        // Check if there are valid farmers
        if valid_farmers.is_empty() {
            return Err(StabilizationError::FarmerNotRegistered);
        }
        
        // Check if fund has sufficient balance
        let total_payout_needed = price_difference
            .checked_mul(total_capacity)
            .ok_or(StabilizationError::InvalidInput)?;
        if fund.total_balance < total_payout_needed {
            return Err(StabilizationError::InsufficientFunds);
        }
        
        // Create distribution map
        let mut distribution = Map::new(&env);
        let timestamp = env.ledger().timestamp();
        
        // Distribute funds to farmers
        for farmer_address in valid_farmers.iter() {
            let farmer_crop_key = DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            let farmer_crop: FarmerCrop = env.storage().persistent().get(&farmer_crop_key).unwrap();
            
            // Calculate farmer's share
            let farmer_share = farmer_crop.production_capacity
                .checked_mul(total_payout_needed)
                .and_then(|product| product.checked_div(total_capacity))
                .ok_or(StabilizationError::InvalidInput)?;
            
            // Update farmer's total received payouts
            let farmer_key = DataKey::Farmer(farmer_address.clone());
            let mut farmer: Farmer = env.storage().persistent().get(&farmer_key).unwrap();
            farmer.total_received_payouts = farmer.total_received_payouts
                .checked_add(farmer_share)
                .ok_or(StabilizationError::InvalidInput)?;
            env.storage().persistent().set(&farmer_key, &farmer);
            
            // Record the payout
            let payout_counter_key = DataKey::PayoutCounter(fund_id.clone(), farmer_address.clone());
            let payout_counter = if env.storage().persistent().has(&payout_counter_key) {
                let counter: u64 = env.storage().persistent().get(&payout_counter_key).unwrap();
                counter.checked_add(1).ok_or(StabilizationError::InvalidInput)?
            } else {
                1
            };
            
            let payout = Payout {
                farmer_id: farmer_address.clone(),
                fund_id: fund_id.clone(),
                amount: farmer_share,
                timestamp,
                market_price: price_data.price,
                threshold_price: fund.price_threshold,
            };
            
            let payout_key = DataKey::Payout(fund_id.clone(), farmer_address.clone(), payout_counter);
            env.storage().persistent().set(&payout_key, &payout);
            env.storage().persistent().set(&payout_counter_key, &payout_counter);
            
            // Add to distribution map
            distribution.set(farmer_address.clone(), farmer_share);
        }
        
        // Update fund balance and last payout time
        fund.total_balance -= total_payout_needed;
        fund.last_payout_time = Some(timestamp);
        env.storage().persistent().set(&fund_key, &fund);
        
        Ok(distribution)
    }

    fn register_farmer(
        env: Env,
        farmer: Address,
        crop_type: String,
        production_capacity: i128,
    ) -> Result<(), StabilizationError> {
        // Verify farmer authorization
        farmer.require_auth();
        
        // Validate inputs
        if crop_type.is_empty() || production_capacity <= 0 {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Register farmer if not already registered
        let farmer_key = DataKey::Farmer(farmer.clone());
        let timestamp = env.ledger().timestamp();
        
        if !env.storage().persistent().has(&farmer_key) {
            let farmer_data = Farmer {
                address: farmer.clone(),
                registered_time: timestamp,
                total_received_payouts: 0,
                active: true,
            };
            env.storage().persistent().set(&farmer_key, &farmer_data);
        }
        
        // Register farmer's crop
        let farmer_crop_key = DataKey::FarmerCrops(farmer.clone(), crop_type.clone());
        
        // Check if farmer already registered this crop
        if env.storage().persistent().has(&farmer_crop_key) {
            return Err(StabilizationError::FarmerAlreadyRegistered);
        }
        
        let farmer_crop = FarmerCrop {
            crop_type: crop_type.clone(),
            production_capacity,
        };
        
        env.storage().persistent().set(&farmer_crop_key, &farmer_crop);
        
        Ok(())
    }

    fn get_payout_history(
        env: Env,
        farmer: Address,
        fund_id: BytesN<32>,
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
        
        // Get payout counter
        let payout_counter_key = DataKey::PayoutCounter(fund_id.clone(), farmer.clone());
        if !env.storage().persistent().has(&payout_counter_key) {
            // No payouts yet
            return Ok(Vec::new(&env));
        }
        
        let payout_counter: u64 = env.storage().persistent().get(&payout_counter_key).unwrap();
        
        // Collect payout history
        let mut history = Vec::new(&env);
        
        for i in 1..=payout_counter {
            let payout_key = DataKey::Payout(fund_id.clone(), farmer.clone(), i);
            if env.storage().persistent().has(&payout_key) {
                let payout: Payout = env.storage().persistent().get(&payout_key).unwrap();
                
                let mut payout_map = Map::new(&env);
                payout_map.set(String::from_str(&env, "amount"), payout.amount);
                payout_map.set(String::from_str(&env, "timestamp"), payout.timestamp as i128);
                payout_map.set(String::from_str(&env, "market_price"), payout.market_price);
                payout_map.set(String::from_str(&env, "threshold_price"), payout.threshold_price);
                
                history.push_back(payout_map);
            }
        }
        
        Ok(history)
    }

    fn calculate_payout_amount(
        env: Env,
        farmer: Address,
        fund_id: BytesN<32>,
    ) -> Result<i128, StabilizationError> {
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
        
        let fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();
        
        // Check if farmer grows this crop type
        let farmer_crop_key = DataKey::FarmerCrops(farmer.clone(), fund.crop_type.clone());
        if !env.storage().persistent().has(&farmer_crop_key) {
            return Err(StabilizationError::FarmerNotRegistered);
        }
        
        // Get the current market price
        let price_key = DataKey::MarketPrice(fund.crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }
        
        let price_data: PriceData = env.storage().persistent().get(&price_key).unwrap();
        
        // Check if price is below threshold
        if price_data.price >= fund.price_threshold {
            return Ok(0); // No payout needed
        }
        
        // Calculate the price difference
        let price_difference = fund.price_threshold - price_data.price;
        
        // Get farmer's production capacity
        let farmer_crop: FarmerCrop = env.storage().persistent().get(&farmer_crop_key).unwrap();
        
        // Calculate potential payout
        let potential_payout = price_difference
            .checked_mul(farmer_crop.production_capacity)
            .ok_or(StabilizationError::InvalidInput)?;
        
        Ok(potential_payout)
    }
}