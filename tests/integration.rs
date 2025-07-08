//! Integration tests for BIP-353 Integrations

use bip353::{Bip353Resolver, ResolverConfig, PaymentType};
use std::time::Duration;
use tokio::runtime::Runtime;

// Most of these tests require actual DNS resolution and real BIP-353 records,
// so they're ignored by default. Run them with `cargo test -- --ignored`.

#[test]
fn test_parse_address() {
    // Regular user@domain format
    let result = bip353::parse_address("alice@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "alice");
    assert_eq!(domain, "example.com");
    
    // With Bitcoin prefix
    let result = bip353::parse_address("₿bob@bitcoin.org");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "bob");
    assert_eq!(domain, "bitcoin.org");
    
    // With whitespace
    let result = bip353::parse_address("  charlie@example.org  ");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "charlie");
    assert_eq!(domain, "example.org");
    
    // With subdomain
    let result = bip353::parse_address("dave@subdomain.example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "dave");
    assert_eq!(domain, "subdomain.example.com");
    
    // With numbers and special chars in user part
    let result = bip353::parse_address("user123_456@example.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "user123_456");
    assert_eq!(domain, "example.com");
    
    // With dash in domain
    let result = bip353::parse_address("eve@example-domain.com");
    assert!(result.is_ok());
    let (user, domain) = result.unwrap();
    assert_eq!(user, "eve");
    assert_eq!(domain, "example-domain.com");
}

#[test]
fn test_invalid_addresses() {
    // Missing @
    let result = bip353::parse_address("aliceexample.com");
    assert!(result.is_err());
    
    // Empty user part
    let result = bip353::parse_address("@example.com");
    assert!(result.is_err());
    
    // Empty domain part
    let result = bip353::parse_address("alice@");
    assert!(result.is_err());
    
    // Multiple @ symbols
    let result = bip353::parse_address("alice@example@com");
    assert!(result.is_err());
    
    // Empty string
    let result = bip353::parse_address("");
    assert!(result.is_err());
    
    // Only whitespace
    let result = bip353::parse_address("   ");
    assert!(result.is_err());
}

#[test]
fn test_resolver_creation() {
    // Default configuration
    let resolver = Bip353Resolver::new();
    assert!(resolver.is_ok());
    
    // Custom configuration
    let config = ResolverConfig::default()
        .with_timeout(Duration::from_secs(10))
        .with_dns_resolver("8.8.8.8:53".parse().unwrap());
    
    let resolver = Bip353Resolver::with_config(config);
    assert!(resolver.is_ok());
    
    // Testnet configuration
    let config = ResolverConfig::testnet();
    let resolver = Bip353Resolver::with_config(config);
    assert!(resolver.is_ok());
}

// Test with a short timeout to ensure it fails fast
#[test]
fn test_timeout() {
    // Create the runtime
    let rt = Runtime::new().unwrap();
    
    // Create the resolver with a very short timeout
    let config = ResolverConfig::default()
        .with_timeout(Duration::from_millis(1)); // 1ms timeout
    
    let resolver = Bip353Resolver::with_config(config).unwrap();
    
    // Try to resolve - should fail quickly
    let start = std::time::Instant::now();
    let result = rt.block_on(async {
        resolver.resolve("test", "example.com").await
    });
    
    // The operation should complete in a reasonable time (much less than 5 seconds)
    assert!(start.elapsed() < Duration::from_secs(5));
    
    // The result should be an error
    assert!(result.is_err());
}

// Test network-specific configuration
#[test]
fn test_network_config() {
    // Create testnet resolver
    let config = ResolverConfig::testnet();
    let _resolver = Bip353Resolver::with_config(config).unwrap();
    
    // Create signet resolver
    let config = ResolverConfig::signet();
    let _resolver = Bip353Resolver::with_config(config).unwrap();
    
    // Create regtest resolver
    let config = ResolverConfig::regtest();
    let _resolver = Bip353Resolver::with_config(config).unwrap();
}

// Test payment type enum functionality
#[test]
fn test_payment_type_display() {
    assert_eq!(PaymentType::OnChain.to_string(), "on-chain");
    assert_eq!(PaymentType::Lightning.to_string(), "lightning");
    assert_eq!(PaymentType::LightningOffer.to_string(), "lightning-offer");
    assert_eq!(PaymentType::Unknown.to_string(), "unknown");
}

// Test invalid domain (this test doesn't require a real network connection)
#[test]
fn test_invalid_domain() {
    // Create the runtime
    let rt = Runtime::new().unwrap();
    
    // Create the resolver
    let resolver = Bip353Resolver::new().unwrap();
    
    // Resolve a domain that's not supposed to exist
    let result = rt.block_on(async {
        resolver.resolve("nonexistent", "example.invalid").await
    });
    
    // Should return an error
    assert!(result.is_err());
}

// Requires a real BIP-353 address to test against
// This test is ignored by default and only runs when explicitly enabled
#[test]
#[ignore]
fn test_real_bip353_address() {
    // Create the runtime
    let rt = Runtime::new().unwrap();
    
    // Create the resolver
    let resolver = Bip353Resolver::new().unwrap();
    
    // Try to resolve a potentially valid BIP-353 address
    // Note: This will likely fail unless you have a known working BIP-353 address
    let result = rt.block_on(async {
        resolver.resolve("test", "mattcorallo.com").await
    });
    
    // The test passes if either it succeeds or fails gracefully
    match result {
        Ok(info) => {
            // If successful, verify the URI format
            assert!(info.uri.starts_with("bitcoin:"));
            println!("Successfully resolved: {}", info.uri);
        }
        Err(e) => {
            // If it fails, that's expected for most addresses
            println!("Expected failure for test address: {}", e);
        }
    }
}