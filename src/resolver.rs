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
    metrics::Bip353Metrics,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, Duration};

/// Type of resolver to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverType {
    /// DNS resolver using DNS-over-TCP
    DNS,
    
    /// HTTP resolver using HTTPS
    #[cfg(feature = "http")]
    HTTP,
}

/// Enhanced payment info with safety warnings
#[derive(Debug, Clone)]
pub struct SafePaymentInfo {
    pub payment_info: PaymentInfo,
    pub warnings: Vec<AddressWarning>,
    pub last_checked: SystemTime,
}

/// Address usage warning
#[derive(Debug, Clone)]
pub enum AddressWarning {
    /// Address was used in a previous transaction
    AddressReused { tx_id: String },
    /// DNS record is stale
    StaleRecord { age: Duration },
    /// DNSSEC validation issues
    DnssecWarning { message: String },
}

/// Simple address cache with TTL
#[derive(Debug)]
struct AddressCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    payment_info: PaymentInfo,
    cached_at: SystemTime,
    ttl: Duration,
}

impl AddressCache {
    fn new(default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }
    
    async fn get(&self, hrn: &str) -> Option<PaymentInfo> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(hrn) {
            if entry.cached_at.elapsed().unwrap_or(Duration::MAX) < entry.ttl {
                return Some(entry.payment_info.clone());
            }
        }
        None
    }
    
    async fn insert(&self, hrn: String, payment_info: PaymentInfo) {
        let mut entries = self.entries.write().await;
        entries.insert(hrn, CacheEntry {
            payment_info,
            cached_at: SystemTime::now(),
            ttl: self.default_ttl,
        });
    }
    
    async fn invalidate(&self, hrn: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(hrn);
    }
}

/// BIP-353 resolver - (what's actually needed)
pub struct Bip353Resolver {
    dns_resolver: DNSHrnResolver,
    #[cfg(feature = "http")]
    http_resolver: HTTPHrnResolver,
    resolver_type: ResolverType,
    config: ResolverConfig,
    cache: Option<Arc<AddressCache>>,
    metrics: Option<Arc<Bip353Metrics>>,
    // Removed: chain_monitor here (not used yet but will be considered in later versions)
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
            cache: None,        
            metrics: None,      
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
            cache: None,        
            metrics: None,      
        })
    }
    
    /// Create a new resolver with enhanced features (only cache and metrics)
    pub fn with_enhanced_config(
        config: ResolverConfig,
        enable_cache: bool,
        cache_ttl: Duration,
        enable_metrics: bool,
    ) -> Result<Self, Bip353Error> {
        let cache = if enable_cache {
            Some(Arc::new(AddressCache::new(cache_ttl)))
        } else {
            None
        };
        
        let metrics = if enable_metrics {
            Some(Arc::new(Bip353Metrics::new()))
        } else {
            None
        };
        
        Ok(Self { 
            dns_resolver: DNSHrnResolver(config.dns_resolver),
            #[cfg(feature = "http")]
            http_resolver: HTTPHrnResolver,
            resolver_type: ResolverType::DNS,
            config,
            cache,
            metrics,
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
    
    /// Resolve with basic safety checks (cache + warnings)
    pub async fn resolve_with_safety_checks(&self, user: &str, domain: &str) -> Result<SafePaymentInfo, Bip353Error> {
        let hrn = format!("{}@{}", user, domain);
        
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(&hrn).await {
                // Record cache hit
                if let Some(metrics) = &self.metrics {
                    metrics.record_cache_hit();
                }
                
                return Ok(SafePaymentInfo {
                    payment_info: cached,
                    warnings: vec![], // No warnings for cached results for now
                    last_checked: SystemTime::now(),
                });
            } else if let Some(metrics) = &self.metrics {
                metrics.record_cache_miss();
            }
        }
        
        // Resolve using main impl
        let start_time = std::time::Instant::now();
        let payment_info = self.resolve(user, domain).await?;
        let resolution_time = start_time.elapsed();
        
        // Cache the result
        if let Some(cache) = &self.cache {
            cache.insert(hrn.clone(), payment_info.clone()).await;
        }
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_resolution_success(domain, resolution_time).await;
        }
        
        // Basic warnings (can be extended later)
        let warnings = self.check_basic_warnings(&payment_info).await;
        
        Ok(SafePaymentInfo {
            payment_info,
            warnings,
            last_checked: SystemTime::now(),
        })
    }
    
    /// Basic warning checks that don't require blockchain integration
    async fn check_basic_warnings(&self, _payment_info: &PaymentInfo) -> Vec<AddressWarning> {
        let warnings = vec![];
        
        // Future: Adding basic checks like:
        // - URI format validation
        // - Parameter validation
        // - Network compatibility checks
        
        warnings
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            let mut entries = cache.entries.write().await;
            entries.clear();
        }
    }
    
    /// Invalidate specific cache entry
    pub async fn invalidate_cache(&self, hrn: &str) {
        if let Some(cache) = &self.cache {
            cache.invalidate(hrn).await;
        }
    }
    
    /// Get metrics if enabled
    pub fn get_metrics(&self) -> Option<crate::metrics::ResolutionStats> {
        self.metrics.as_ref().map(|m| m.get_resolution_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore]
    async fn test_resolve_address() {
        let resolver = Bip353Resolver::new().unwrap();
        let result = resolver.resolve_address("user@example.com").await;
        
        if result.is_ok() {
            let info = result.unwrap();
            assert!(info.uri.starts_with("bitcoin:"));
        }
    }
    
    #[tokio::test]
    async fn test_enhanced_resolver() {
        let config = ResolverConfig::default();
        let resolver = Bip353Resolver::with_enhanced_config(
            config,
            true, // enable cache
            Duration::from_secs(300), // cache TTL
            true, // enable metrics
        ).unwrap();
        
        // Test cache functionality
        resolver.clear_cache().await;
        resolver.invalidate_cache("test@example.com").await;
        
        // Verify enhanced features are enabled
        assert!(resolver.cache.is_some());
        assert!(resolver.metrics.is_some());
        assert!(resolver.get_metrics().is_some());
    }
}