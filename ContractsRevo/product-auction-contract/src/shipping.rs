use soroban_sdk::{contractimpl, Address, Env, String, Symbol, Vec};

use crate::{datatype::{DataKeys, Shipment, ShippingError, COST_PER_KM, COST_PER_POUND}, interfaces::ShippingOperations, ProductAuctionContract, ProductAuctionContractArgs, ProductAuctionContractClient};

#[contractimpl]
impl ShippingOperations for ProductAuctionContract {
    fn calculate_shipping_cost(weight_pounds: u32, distance_km: u32) -> u64 {
        
        let weight_cost = weight_pounds as u64 * COST_PER_POUND;
        let distance_cost = distance_km as u64 * COST_PER_KM;
        weight_cost + distance_cost
    }

    fn estimate_delivery_time(distance_km: u32) -> u32 {
        match distance_km {
            0..=50 => 1,
            51..=200 => 3,
            201..=500 => 5,
            _ => 7,
        }
    }

    fn create_shipment(
        env: Env,
        seller: Address,
        buyer: Address,
        buyer_zone: String,
        weight_pounds: u32,
        distance_km: u32,
        tracking_number: String
    ) -> Result<String, ShippingError> {
        seller.require_auth();

        let shipment_key = DataKeys::Shipment(seller.clone(), tracking_number.clone());

        // Ensure shipment does not already exist
        if env.storage().persistent().has(&shipment_key) {
            return Err(ShippingError::ShipmentAlreadyExists);
        }

        if buyer_zone.is_empty() {
            return Err(ShippingError::InvalidBuyerZone);
        }        

        // Ensure the buyer is in a restricted zone
        let restricted_locations = Vec::from_array(&env, [
            String::from_str(&env, "RestrictedZone1"),
            String::from_str(&env, "RestrictedZone2"), //Could this be a list of restricted zones that is stored in the contract?
        ]);
        
        if restricted_locations.iter().any(|location| location == buyer_zone) {
            return Err(ShippingError::RestrictedLocation);
        }

        let shipping_cost = Self::calculate_shipping_cost(weight_pounds, distance_km);
        let delivery_time = Self::estimate_delivery_time(distance_km);

        //Create new shipment
        let shipment = Shipment {
            seller: seller.clone(),
            buyer: buyer.clone(),
            weight_pounds,
            distance_km,
            shipping_cost,
            delivery_estimate_days: delivery_time,
            status: Symbol::new(&env, "Pending"),
            tracking_number: tracking_number.clone(),
        };

        // Retrieve or initialize the shipment list for the seller
        let key = DataKeys::ShipmentList(seller.clone());
        let mut shipments = env
            .storage()
            .persistent()
            .get::<_, Vec<Shipment>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        // Add the new shipment to the list
        shipments.push_back(shipment.clone());

        // Save the updated shipment list
        env.storage().persistent().set(&key, &shipments);

        // Save the individual shipment under its own key
        env.storage().persistent().set(&shipment_key, &shipment);

        env.events().publish(("ShipmentCreated", tracking_number.clone()), shipment.clone());

        Ok(tracking_number)
    }

    fn update_shipping_status(env: Env, tracking_number: String, seller: Address, new_status: Symbol) -> Result<(), ShippingError> {
        seller.require_auth();

        let shipment_key = DataKeys::Shipment(seller, tracking_number.clone());
        let mut shipment: Shipment = env
            .storage()
            .instance()
            .get(&shipment_key)
            .ok_or(ShippingError::ShipmentNotFound)?;

        shipment.status = new_status.clone();
        env.storage().instance().set(&shipment_key, &shipment);

        env.events().publish(("StatusUpdated", tracking_number), new_status);

        Ok(())
    }
}