use soroban_sdk::{contractimpl, Address, Env, String, Symbol, Vec};
use uuid::Uuid;

use crate::{datatype::{Condition, DataKeys, Product, ProductError}, interfaces::ProductListing, ProductAuctionContract, ProductAuctionContractArgs, ProductAuctionContractClient};


#[contractimpl]
impl ProductListing for ProductAuctionContract {
    fn add_product(
        env: Env,
        seller: Address,
        name: Symbol,
        description: String,
        price: u64,
        condition: Condition,
        stock: u32,
        images: Vec<String>,
        weight_grams: u64,
    ) -> Result<(), ProductError> {
        // Ensure the seller is authorized
        seller.require_auth();

        // Validate description length (between 10 - 500 chars)
        if description.len() < 10 || description.len() > 500 {
            return Err(ProductError::InvalidDescription);
        }

        // Validate price is not zero
        if price == 0 {
            return Err(ProductError::InvalidPrice);
        }

        // Ensure there is at least one image and not more than 10
        if images.is_empty() || images.len() > 10 {
            return Err(ProductError::InvalidImageCount);
        }

        // Validate product weight (must be > 0)
        if weight_grams == 0 {
            return Err(ProductError::InvalidWeight);
        }

        // Generate a unique product ID using UUID
        let product_id = Uuid::new_v4().as_u128();

        // Create the product
        let product = Product {
            id: product_id,
            seller: seller.clone(),
            name: name.clone(),
            description,
            price,
            condition,
            stock,
            images,
            weight_grams,
        };

        // Retrieve or initialize the product list for the seller
        let key = DataKeys::ProductList(seller.clone());
        let mut products = env
            .storage()
            .persistent()
            .get::<_, Vec<Product>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        // Add the new product to the list
        products.push_back(product.clone());

        // Save the updated product list
        env.storage().persistent().set(&key, &products);

        // Save the individual product under its own key
        let product_key = DataKeys::Product(seller.clone(), product_id);
        env.storage().persistent().set(&product_key, &product);

        // Emit an event for the new product
        env.events().publish(
            (seller.clone(), "ProductAdded", name.clone()),
            product.clone(),
        );

        Ok(())
    }

    fn get_products(env: Env, seller: Address) -> Result<Vec<Product>, ProductError> {
        let key = DataKeys::ProductList(seller.clone());

        let products = env
            .storage()
            .persistent()
            .get::<_, Vec<Product>>(&key)
            .unwrap_or_else(||
                Vec::new(&env),
            );
        
        return Ok(products);
    }

    fn get_product(env: Env, seller: Address, product_id: u128) -> Result<Product, ProductError> {
        let key = DataKeys::Product(seller.clone(), product_id);

        let product = env.storage()
            .instance()
            .get(&key)
            .ok_or(ProductError::ProductNotFound)?;

        Ok(product)
    }

    fn update_stock(env: Env, seller: Address, product_id: u128, new_stock: u32) -> Result<(), ProductError> {
        seller.require_auth();

        let key = DataKeys::Product(seller.clone(), product_id);

        let mut product: Product = env.storage()
            .instance()
            .get(&key)
            .ok_or(ProductError::ProductNotFound)?;


        product.stock = new_stock;
        env.storage().instance().set(&key, &product);
        
        Ok(())
    }
}