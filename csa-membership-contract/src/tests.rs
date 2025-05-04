#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Env, Address, String};

    #[test]
    fn test_valid_enrollment() {
        let env = Env::default();
        let admin = Address::random(&env);
        let member = Address::random(&env);

        // Initialize the contract
        let contract = CSAMembershipContract::init(env.clone(), admin.clone());

        // Enroll in Summer 2025 CSA with Medium share
        let season = String::from("Summer 2025");
        let share_size = String::from("Medium");
        let pickup_location = String::from("Downtown Market");

        let result = contract.enroll_member(
            env.clone(),
            member.clone(),
            season.clone(),
            share_size.clone(),
            pickup_location.clone(),
        );

        assert!(result.is_ok());

        // Verify token minting and metadata
        let token = contract.get_membership_token(env.clone(), member.clone(), season.clone());
        assert_eq!(token.share_size, share_size);
        assert_eq!(token.pickup_location, pickup_location);
    }

    #[test]
    fn test_pickup_location_update() {
        let env = Env::default();
        let admin = Address::random(&env);
        let member = Address::random(&env);

        // Initialize the contract and enroll a member
        let contract = CSAMembershipContract::init(env.clone(), admin.clone());
        let season = String::from("Summer 2025");
        let share_size = String::from("Medium");
        let pickup_location = String::from("Downtown Market");

        contract
            .enroll_member(
                env.clone(),
                member.clone(),
                season.clone(),
                share_size.clone(),
                pickup_location.clone(),
            )
            .unwrap();

        // Update pickup location
        let new_pickup_location = String::from("City Center");
        let result = contract.update_pickup_location(
            env.clone(),
            member.clone(),
            season.clone(),
            new_pickup_location.clone(),
        );

        assert!(result.is_ok());

        // Verify the updated pickup location
        let token = contract.get_membership_token(env.clone(), member.clone(), season.clone());
        assert_eq!(token.pickup_location, new_pickup_location);
    }

    #[test]
    fn test_membership_cancellation() {
        let env = Env::default();
        let admin = Address::random(&env);
        let member = Address::random(&env);

        // Initialize the contract and enroll a member
        let contract = CSAMembershipContract::init(env.clone(), admin.clone());
        let season = String::from("Summer 2025");
        let share_size = String::from("Medium");
        let pickup_location = String::from("Downtown Market");

        contract
            .enroll_member(
                env.clone(),
                member.clone(),
                season.clone(),
                share_size.clone(),
                pickup_location.clone(),
            )
            .unwrap();

        // Cancel membership before the start date
        let result = contract.cancel_membership(env.clone(), member.clone(), season.clone());
        assert!(result.is_ok());

        // Verify membership cancellation
        let token = contract.get_membership_token(env.clone(), member.clone(), season.clone());
        assert!(token.is_none());
    }

    #[test]
    fn test_membership_transfer() {
        let env = Env::default();
        let admin = Address::random(&env);
        let member = Address::random(&env);
        let new_member = Address::random(&env);

        // Initialize the contract and enroll a member
        let contract = CSAMembershipContract::init(env.clone(), admin.clone());
        let season = String::from("Summer 2025");
        let share_size = String::from("Medium");
        let pickup_location = String::from("Downtown Market");

        contract
            .enroll_member(
                env.clone(),
                member.clone(),
                season.clone(),
                share_size.clone(),
                pickup_location.clone(),
            )
            .unwrap();

        // Transfer membership to a new member
        let result = contract.transfer_membership(
            env.clone(),
            member.clone(),
            new_member.clone(),
            season.clone(),
        );

        assert!(result.is_ok());

        // Verify the new member has the token
        let token = contract.get_membership_token(env.clone(), new_member.clone(), season.clone());
        assert!(token.is_some());
        assert_eq!(token.unwrap().share_size, share_size);
    }
}
