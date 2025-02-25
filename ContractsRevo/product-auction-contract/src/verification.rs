use soroban_sdk::{contractimpl, Address, Env, String, Symbol};

use crate::{datatype::{Condition, DataKeys, Dispute, DisputeStatus, Product, ReturnRequest, SellerVerificationStatus, VerificationError}, interfaces::VerificationOperations, ProductAuctionContract, ProductAuctionContractArgs, ProductAuctionContractClient};

#[contractimpl]
impl VerificationOperations for ProductAuctionContract {
    fn verify_product(env: Env, admin: Address, seller: Address, product_id: u128, is_authentic: bool) -> Result<(), VerificationError> {
        admin.require_auth();

        let product_key = DataKeys::Product(seller.clone(), product_id);
        let mut product: Product = env.storage().persistent().get(&product_key)
            .ok_or(VerificationError::ProductNotFound)?;

        product.verified = is_authentic;
        env.storage().persistent().set(&product_key, &product);

        env.events().publish(("ProductVerified", product_id), is_authentic);

        Ok(())
    }

    fn request_seller_verification(env: Env, seller: Address) -> Result<(), VerificationError> {
        seller.require_auth();

        let verification_key = DataKeys::SellerVerification(seller.clone());

        if env.storage().persistent().has(&verification_key) {
            return Err(VerificationError::AlreadyRequested);
        }

        env.storage().persistent().set(&verification_key, &SellerVerificationStatus::Pending);

        env.events().publish(("SellerVerificationRequested", seller.clone()), ());

        Ok(())
    }

    fn verify_seller(env: Env, admin: Address, seller: Address, is_verified: bool) -> Result<(), VerificationError> {
        admin.require_auth();

        let verification_key = DataKeys::SellerVerification(seller.clone());

        if !env.storage().persistent().has(&verification_key) {
            return Err(VerificationError::NoVerificationRequest);
        }

        let status = if is_verified {
            SellerVerificationStatus::Verified
        } else {
            SellerVerificationStatus::Rejected
        };

        env.storage().persistent().set(&verification_key, &status);

        env.events().publish(("SellerVerified", seller.clone()), status);

        Ok(())
    }

    fn verify_condition(env: Env, admin: Address, seller: Address, product_id: u128, condition: Condition) -> Result<(), VerificationError> {
        admin.require_auth();

        let product_key = DataKeys::Product(seller.clone(), product_id);
        let mut product: Product = env.storage().persistent().get(&product_key)
            .ok_or(VerificationError::ProductNotFound)?;

        product.condition = condition.clone();
        env.storage().persistent().set(&product_key, &product);

        env.events().publish(("ConditionVerified", product_id), condition);

        Ok(())
    }

    fn open_dispute(env: Env, buyer: Address, seller: Address, product_id: u128, reason: String) -> Result<(), VerificationError> {
        buyer.require_auth();

        let dispute_key = DataKeys::Dispute(buyer.clone(), seller.clone(), product_id);

        if env.storage().persistent().has(&dispute_key) {
            return Err(VerificationError::DisputeAlreadyExists);
        }

        let dispute = Dispute {
            buyer: buyer.clone(),
            seller: seller.clone(),
            product_id,
            reason,
            status: DisputeStatus::Pending,
        };

        env.storage().persistent().set(&dispute_key, &dispute);

        env.events().publish(("DisputeOpened", product_id), dispute.clone());

        Ok(())
    }

    fn resolve_dispute(env: Env, admin: Address, buyer: Address, seller: Address, product_id: u128, resolution: DisputeStatus) -> Result<(), VerificationError> {
        admin.require_auth();

        let dispute_key = DataKeys::Dispute(buyer.clone(), seller.clone(), product_id);
        let mut dispute: Dispute = env.storage().persistent().get(&dispute_key)
            .ok_or(VerificationError::DisputeNotFound)?;

        dispute.status = resolution.clone();
        env.storage().persistent().set(&dispute_key, &dispute);

        env.events().publish(("DisputeResolved", product_id), resolution);

        Ok(())
    }

    fn set_return_policy(env: Env, seller: Address, policy: String) -> Result<(), VerificationError> {
        seller.require_auth();

        let return_key = DataKeys::ReturnPolicy(seller.clone());

        env.storage().persistent().set(&return_key, &policy);

        env.events().publish(("ReturnPolicySet", seller.clone()), policy);

        Ok(())
    }

    fn request_return(env: Env, buyer: Address, seller: Address, product_id: u128, reason: String) -> Result<(), VerificationError> {
        buyer.require_auth();

        let return_key = DataKeys::ReturnRequest(buyer.clone(), product_id);

        if env.storage().persistent().has(&return_key) {
            return Err(VerificationError::ReturnAlreadyRequested);
        }

        let return_request = ReturnRequest {
            buyer: buyer.clone(),
            seller: seller.clone(),
            product_id,
            reason,
            status: Symbol::new(&env, "Requested"),
        };

        env.storage().persistent().set(&return_key, &return_request);

        env.events().publish(("ReturnRequested", product_id), return_request.clone());

        Ok(())
    }

    fn resolve_return(env: Env, admin: Address, buyer: Address, product_id: u128, resolution: Symbol) -> Result<(), VerificationError> {
        admin.require_auth();

        let return_key = DataKeys::ReturnRequest(buyer.clone(), product_id);
        let mut return_request: ReturnRequest = env.storage().persistent().get(&return_key)
            .ok_or(VerificationError::ReturnRequestNotFound)?;

        return_request.status = resolution.clone();
        env.storage().persistent().set(&return_key, &return_request);

        env.events().publish(("ReturnResolved", product_id), resolution);

        Ok(())
    }
}
