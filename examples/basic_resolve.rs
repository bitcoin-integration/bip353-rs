//! A basic example of resolving a BIP-353 address

use bip353::{Bip353Resolver, ResolverConfig};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the address from command line arguments
    let args: Vec<String> = env::args().collect();
    let address = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("Usage: {} <bitcoin-address>", args[0]);
        eprintln!("Example: {} ₿user@example.com", args[0]);
        return Ok(());
    };
    
    // Create a resolver with custom configuration
    let config = ResolverConfig::default()
        .with_timeout(Duration::from_secs(10))
        .with_dns_resolver("8.8.8.8:53".parse()?);
    
    let resolver = Bip353Resolver::with_config(config)?;
    
    // Parse the address to show the user and domain parts
    match bip353::parse_address(address) {
        Ok((user, domain)) => {
            println!("Parsed address:");
            println!("  User:   {}", user);
            println!("  Domain: {}", domain);
            println!();
        },
        Err(err) => {
            eprintln!("Failed to parse address: {}", err);
            return Err(err.into());
        }
    }
    
    // Resolve the address
    println!("Resolving address {}...", address);
    match resolver.resolve_address(address).await {
        Ok(info) => {
            println!("Resolution successful!");
            println!("  URI:      {}", info.uri);
            println!("  Type:     {:?}", info.payment_type);
            println!("  Reusable: {}", info.is_reusable);
            
            if !info.parameters.is_empty() {
                println!("  Parameters:");
                for (key, value) in &info.parameters {
                    println!("    {}: {}", key, value);
                }
            }
        },
        Err(err) => {
            eprintln!("Failed to resolve address: {}", err);
            return Err(err.into());
        }
    }
    
    Ok(())
}