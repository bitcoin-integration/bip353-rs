//! This is useful in monitoring backend events

use bitcoin::Address;
use async_trait::async_trait;

/// (lets users plug in their own)
#[async_trait]
pub trait ChainBackend: Send + Sync {
    /// Check if an address has been used
    async fn is_address_used(&self, address: &Address) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get transaction history for an address
    async fn get_address_history(&self, address: &Address) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Simple event when an address is detected as used
#[derive(Debug, Clone)]
pub struct AddressUsedEvent {
    pub hrn: String,
    pub address: Address,
    pub tx_id: String,
}

/// Simple chain monitor(placeholder)
pub struct ChainMonitor {
    // This is a placeholder(your true chain impl would go here)
    _placeholder: (),
}

impl ChainMonitor {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
    
    /// Check if an address has been used (placeholder)
    pub async fn check_address_usage(&self, _address: &Address) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder - always return false for now
        Ok(false)
    }
}