//! Configuration options for BIP-353 resolver

use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;

/// Configuration for BIP-353 resolver
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// The DNS resolver to use (IP and port)
    pub dns_resolver: SocketAddr,
    
    /// Whether to enforce DNSSEC validation
    pub enforce_dnssec: bool,
    
    /// Timeout for DNS queries in milliseconds
    pub timeout_ms: u64,
    
    /// Whether to allow HTTP resolution fallback 
    /// (for domains that don't support BIP-353 DNS but do support LN-Address)
    pub allow_http_fallback: bool,
    
    /// Network to use for parsing payment instructions
    pub network: bitcoin::Network,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            // Google DNS with DNSSEC support
            dns_resolver: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53),
            enforce_dnssec: true,
            timeout_ms: 5000, // 5 second timeout
            allow_http_fallback: true,
            network: bitcoin::Network::Bitcoin,
        }
    }
}

impl ResolverConfig {
    /// Create a configuration for testnet
    pub fn testnet() -> Self {
        Self {
            network: bitcoin::Network::Testnet,
            ..Default::default()
        }
    }
    
    /// Create a configuration for signet
    pub fn signet() -> Self {
        Self {
            network: bitcoin::Network::Signet, 
            ..Default::default()
        }
    }

    /// Create a configuration for regtest
    pub fn regtest() -> Self {
        Self {
            network: bitcoin::Network::Regtest,
            ..Default::default()
        }
    }
    
    /// Set the DNS resolver
    pub fn with_dns_resolver(mut self, resolver: SocketAddr) -> Self {
        self.dns_resolver = resolver;
        self
    }
    
    /// Set the DNSSEC enforcement
    pub fn with_dnssec(mut self, enforce: bool) -> Self {
        self.enforce_dnssec = enforce;
        self
    }
    
    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_ms = timeout.as_millis() as u64;
        self
    }
    
    /// Set whether to allow HTTP fallback
    pub fn with_http_fallback(mut self, allow: bool) -> Self {
        self.allow_http_fallback = allow;
        self
    }
    
    /// Set the network
    pub fn with_network(mut self, network: bitcoin::Network) -> Self {
        self.network = network;
        self
    }
    
    /// Get the timeout as a Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}