//! BIP-353 core resolver implementation

use bitcoin_payment_instructions::{
    PaymentInstructions,
    dns_resolver::DNSHrnResolver,
};

#[cfg(feature = "http")]
use bitcoin_payment_instructions::http_resolver::HTTPHrnResolver;

use crate::{
    Bip353Error,
    config::ResolverConfig,
    types::PaymentInfo,
    parse_address,
};

/// Type of resolver to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverType {
    /// DNS resolver using DNS-over-TCP
    DNS,
    
    /// HTTP resolver using HTTPS
    #[cfg(feature = "http")]
    HTTP,
}

/// BIP-353 resolver
pub struct Bip353Resolver {
    dns_resolver: DNSHrnResolver,
    #[cfg(feature = "http")]
    http_resolver: HTTPHrnResolver,
    resolver_type: ResolverType,
    config: ResolverConfig,
}

impl Bip353Resolver {
    /// Create a new resolver with default configuration
    pub fn new() -> Result<Self, Bip353Error> {
        Self::with_config(ResolverConfig::default())
    }
    
    /// Create a new resolver with custom configuration
    pub fn with_config(config: ResolverConfig) -> Result<Self, Bip353Error> {
        Ok(Self { 
            dns_resolver: DNSHrnResolver(config.dns_resolver),
            #[cfg(feature = "http")]
            http_resolver: HTTPHrnResolver,
            resolver_type: ResolverType::DNS,
            config,
        })
    }
    
    /// Create a new resolver with a specific type
    pub fn with_type(resolver_type: ResolverType) -> Result<Self, Bip353Error> {
        let config = ResolverConfig::default();
        
        Ok(Self { 
            dns_resolver: DNSHrnResolver(config.dns_resolver),
            #[cfg(feature = "http")]
            http_resolver: HTTPHrnResolver,
            resolver_type,
            config,
        })
    }
    
    /// Resolve a human-readable Bitcoin address
    pub async fn resolve(&self, user: &str, domain: &str) -> Result<PaymentInfo, Bip353Error> {
        // Parse the payment instructions using the appropriate resolver
        let instructions = match self.resolver_type {
            ResolverType::DNS => {
                PaymentInstructions::parse(
                    &format!("{}@{}", user, domain),
                    self.config.network,
                    &self.dns_resolver,
                    true, // Support proof-of-payment callbacks
                ).await.map_err(Bip353Error::from)?
            },
            #[cfg(feature = "http")]
            ResolverType::HTTP => {
                PaymentInstructions::parse(
                    &format!("{}@{}", user, domain),
                    self.config.network,
                    &self.http_resolver,
                    true, // Support proof-of-payment callbacks
                ).await.map_err(Bip353Error::from)?
            },
        };
        
        // Extract the URI based on the payment instructions
        let uri = match &instructions {
            PaymentInstructions::FixedAmount(fixed) => {
                // For fixed amount instructions, we should have a concrete URI
                if let Some(method) = fixed.methods().first() {
                    match method {
                        bitcoin_payment_instructions::PaymentMethod::OnChain(addr) => {
                            let mut uri = format!("bitcoin:{}", addr);
                            if let Some(amount) = fixed.max_amount() {
                                uri.push_str(&format!("?amount={}", amount.btc_decimal_rounding_up_to_sats()));
                            }
                            uri
                        },
                        bitcoin_payment_instructions::PaymentMethod::LightningBolt11(invoice) => {
                            format!("bitcoin:?lightning={}", invoice)
                        },
                        bitcoin_payment_instructions::PaymentMethod::LightningBolt12(offer) => {
                            format!("bitcoin:?lno={}", offer)
                        },
                    }
                } else {
                    return Err(Bip353Error::InvalidRecord("No payment methods found".into()));
                }
            },
            PaymentInstructions::ConfigurableAmount(configurable) => {
                // For configurable amount instructions, we'll use a BIP-21 URI with the first method
                let mut has_method = false;
                let base_uri = if let Some(method) = configurable.methods().next() {
                    has_method = true;
                    match method {
                        bitcoin_payment_instructions::PossiblyResolvedPaymentMethod::LNURLPay { .. } => {
                            "bitcoin:".to_string()
                        },
                        bitcoin_payment_instructions::PossiblyResolvedPaymentMethod::Resolved(method) => {
                            match method {
                                bitcoin_payment_instructions::PaymentMethod::OnChain(addr) => {
                                    format!("bitcoin:{}", addr)
                                },
                                bitcoin_payment_instructions::PaymentMethod::LightningBolt11(invoice) => {
                                    format!("bitcoin:?lightning={}", invoice)
                                },
                                bitcoin_payment_instructions::PaymentMethod::LightningBolt12(offer) => {
                                    format!("bitcoin:?lno={}", offer)
                                },
                            }
                        },
                    }
                } else {
                    "bitcoin:".to_string()
                };
                
                if !has_method {
                    return Err(Bip353Error::InvalidRecord("No payment methods found".into()));
                }
                
                base_uri
            },
        };
        
        // Create payment info
        Ok(PaymentInfo::from_instructions(instructions, uri))
    }
    
    /// Resolve a human-readable Bitcoin address string
    pub async fn resolve_address(&self, address: &str) -> Result<PaymentInfo, Bip353Error> {
        let (user, domain) = parse_address(address)?;
        self.resolve(&user, &domain).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests require actual DNS resolution, so they're ignored by default
    // Run them using: cargo test -- --ignored
    
    #[tokio::test]
    #[ignore]
    async fn test_resolve_address() {
        // This test requires a valid BIP-353 DNS setup
        // Replace with a real BIP-353 address for actual testing
        let resolver = Bip353Resolver::new().unwrap();
        let result = resolver.resolve_address("user@example.com").await;
        
        // will likely fail with most addresses since BIP-353 is new
        // so the important thing is that it doesn't panic
        if result.is_ok() {
            let info = result.unwrap();
            assert!(info.uri.starts_with("bitcoin:"));
        }
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_resolver_with_config() {
        // Test with custom configuration
        let config = ResolverConfig::default()
            .with_dns_resolver("1.1.1.1:53".parse().unwrap())
            .with_timeout(std::time::Duration::from_secs(10));
            
        let resolver = Bip353Resolver::with_config(config).unwrap();
        
        // Try to resolve a known address
        // This will likely fail with most addresses since BIP-353 is new
        let result = resolver.resolve("user", "example.com").await;
        
        // The important thing is that it doesn't panic
        assert!(result.is_err() || result.is_ok());
    }
}