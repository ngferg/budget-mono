//! One-shot admin command: creates the "Basic subscription" product and its
//! recurring $5/month Price in Stripe, then prints the identifiers.
//!
//! Run it once per Stripe account (test mode and live mode separately):
//!
//!   STRIPE_SECRET_KEY=sk_test_... cargo run --bin create_subscription_product
//!
//! Copy the printed `STRIPE_PRICE_ID` into the environment the budget REST
//! service runs with; that Price is what Checkout Sessions sell.

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let client = match budget_lib::stripe::StripeClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("{e}");
            eprintln!("Set STRIPE_SECRET_KEY (from the Stripe Dashboard) and try again.");
            std::process::exit(1);
        }
    };

    match client.create_subscription_product().await {
        Ok(product) => {
            println!("Created product : {}", product.product_id);
            println!("Created price   : {}", product.price_id);
            println!();
            println!("Add this to the REST service's environment:");
            println!("  STRIPE_PRICE_ID={}", product.price_id);
        }
        Err(e) => {
            eprintln!("Failed to create the subscription product: {e}");
            std::process::exit(1);
        }
    }
}
