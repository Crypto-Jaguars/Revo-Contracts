#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

mod metadata;
mod mint;
mod proof;

#[contract]
pub struct TransactionNFTContract;

#[contractimpl]
impl TransactionNFTContract {
    pub fn mint_nft(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: u64,
        product: BytesN<32>,
    ) -> BytesN<32> {
        if buyer == seller {
            panic!("Buyer and seller cannot be the same address");
        }

        // Validate amount (must be greater than zero)
        if amount == 0 {
            panic!("Amount must be greater than zero");
        }

        // Require authorization from both buyer and seller
        buyer.require_auth();
        seller.require_auth();

        let product_bytes: soroban_sdk::Bytes = product.clone().into();

        // Check for duplicate transaction before generating proof
        if proof::transaction_exists(&env, &buyer, &seller, amount, &product_bytes) {
            panic!("Duplicate transaction detected");
        }

        let tx_id = proof::generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product_bytes,
        );

        mint::mint_nft(&env, &buyer, tx_id.clone(), &seller, amount, &product);

        tx_id
    }

    pub fn get_nft_metadata(env: Env, tx_id: BytesN<32>) -> Option<mint::NFTMetadata> {
        metadata::get_metadata(&env, &tx_id)
    }
}

#[cfg(test)]
mod tests;
