//! BIP-353 DNS Payment Instructions - Integration Layer
//! 
//! This library provides convenient integration layer around the
//! bitcoin-payment-instructions and dnssec-prover crates for resolving
//! human-readable Bitcoin addresses (₿user@domain) through DNS.
//!
//! It is designed to be easily integrated with Bitcoin Core (through an inbuilt FFI)
//! and Hardware Wallet Interface (via Python bindings).

mod error;
mod resolver;
mod types;
mod config;
mod metrics;     
mod monitoring;   

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "python")]
pub mod python;

pub use error::Bip353Error;
pub use resolver::{Bip353Resolver, ResolverType};
pub use types::{PaymentInfo, PaymentType};
pub use config::ResolverConfig;
pub use metrics::{Bip353Metrics, ResolutionStats, CacheStats};
pub use monitoring::{ChainMonitor, ChainBackend, AddressUsedEvent};

/// BIP-353 Bitcoin address parsing utility
///
/// Parses a human-readable Bitcoin address in the format
/// user@domain or ₿user@domain and returns the user and domain parts.
pub fn parse_address(address: &str) -> Result<(String, String), Bip353Error> {
    let addr = address.trim();
    
    // Remove Bitcoin prefix if present
    let addr = addr.strip_prefix("₿").unwrap_or(addr);
    
    // Split by @
    let parts: Vec<&str> = addr.split('@').collect();
    if parts.len() != 2 {
        return Err(Bip353Error::InvalidAddress("Address must be in format user@domain".into()));
    }
    
    let user = parts[0].trim();
    let domain = parts[1].trim();
    
    if user.is_empty() || domain.is_empty() {
        return Err(Bip353Error::InvalidAddress("User and domain cannot be empty".into()));
    }
    
    Ok((user.to_string(), domain.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        // Regular user@domain format
        let result = parse_address("alice@example.com");
        assert!(result.is_ok());
        let (user, domain) = result.unwrap();
        assert_eq!(user, "alice");
        assert_eq!(domain, "example.com");
        
        // With Bitcoin prefix
        let result = parse_address("₿bob@bitcoin.org");
        assert!(result.is_ok());
        let (user, domain) = result.unwrap();
        assert_eq!(user, "bob");
        assert_eq!(domain, "bitcoin.org");
        
        // With whitespace
        let result = parse_address("  charlie@example.org  ");
        assert!(result.is_ok());
        let (user, domain) = result.unwrap();
        assert_eq!(user, "charlie");
        assert_eq!(domain, "example.org");
    }

    #[test]
    fn test_invalid_addresses() {
        // Missing @
        let result = parse_address("aliceexample.com");
        assert!(result.is_err());
        
        // Empty user part
        let result = parse_address("@example.com");
        assert!(result.is_err());
        
        // Empty domain part
        let result = parse_address("alice@");
        assert!(result.is_err());
        
        // Multiple @ symbols
        let result = parse_address("alice@example@com");
        assert!(result.is_err());
    }
}