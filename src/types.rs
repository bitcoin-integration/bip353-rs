//! Type definitions for BIP-353 integrations

use std::collections::HashMap;
use bitcoin_payment_instructions::{
    PaymentInstructions, 
    PaymentMethod, 
    FixedAmountPaymentInstructions, 
    ConfigurableAmountPaymentInstructions
};

/// Payment instruction type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentType {
    /// On-chain Bitcoin address
    OnChain,
    
    /// Lightning Network invoice (BOLT 11)
    Lightning,
    
    /// Lightning Network offer (BOLT 12)
    LightningOffer,
    
    /// Unknown payment type
    Unknown,
}

impl std::fmt::Display for PaymentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PaymentType::OnChain => write!(f, "on-chain"),
            PaymentType::Lightning => write!(f, "lightning"),
            PaymentType::LightningOffer => write!(f, "lightning-offer"),
            PaymentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Structured payment information
#[derive(Debug, Clone)]
pub struct PaymentInfo {
    /// The Bitcoin URI (BIP-21)
    pub uri: String,
    
    /// The type of payment method
    pub payment_type: PaymentType,
    
    /// Whether the payment address is reusable
    pub is_reusable: bool,
    
    /// Additional parameters from the payment URI
    pub parameters: HashMap<String, String>,
    
    /// Original payment instructions
    pub original_instructions: OriginalInstructions,
}

/// Original payment instructions from the underlying implementation
#[derive(Debug, Clone)]
pub enum OriginalInstructions {
    /// Fixed amount payment instructions
    FixedAmount(FixedAmountPaymentInstructions),
    
    /// Configurable amount payment instructions
    ConfigurableAmount(ConfigurableAmountPaymentInstructions),
}

impl From<PaymentInstructions> for OriginalInstructions {
    fn from(instructions: PaymentInstructions) -> Self {
        match instructions {
            PaymentInstructions::FixedAmount(fixed) => OriginalInstructions::FixedAmount(fixed),
            PaymentInstructions::ConfigurableAmount(configurable) => OriginalInstructions::ConfigurableAmount(configurable),
        }
    }
}

impl PaymentInfo {
    /// Create a new PaymentInfo from PaymentInstructions
    pub fn from_instructions(instructions: PaymentInstructions, uri: String) -> Self {
        let mut parameters = HashMap::new();
        let mut payment_type = PaymentType::Unknown;
        let mut is_reusable = true;
        
        // Parse payment type and reusability
        match &instructions {
            PaymentInstructions::FixedAmount(fixed) => {
                for method in fixed.methods() {
                    match method {
                        PaymentMethod::OnChain(_) => {
                            payment_type = PaymentType::OnChain;
                        },
                        PaymentMethod::LightningBolt11(_) => {
                            payment_type = PaymentType::Lightning;
                            is_reusable = false;
                        },
                        PaymentMethod::LightningBolt12(_) => {
                            payment_type = PaymentType::LightningOffer;
                        },
                    }
                }
            },
            PaymentInstructions::ConfigurableAmount(configurable) => {
                for method in configurable.methods() {
                    match method {
                        bitcoin_payment_instructions::PossiblyResolvedPaymentMethod::LNURLPay { .. } => {
                            payment_type = PaymentType::Lightning;
                            is_reusable = false;
                        },
                        bitcoin_payment_instructions::PossiblyResolvedPaymentMethod::Resolved(method) => {
                            match method {
                                PaymentMethod::OnChain(_) => {
                                    payment_type = PaymentType::OnChain;
                                },
                                PaymentMethod::LightningBolt11(_) => {
                                    payment_type = PaymentType::Lightning;
                                    is_reusable = false;
                                },
                                PaymentMethod::LightningBolt12(_) => {
                                    payment_type = PaymentType::LightningOffer;
                                },
                            }
                        },
                    }
                }
            },
        }
        
        // Parse parameters from URI
        if let Some(query_start) = uri.find('?') {
            let query = &uri[query_start+1..];
            for pair in query.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos+1..].to_string();
                    parameters.insert(key, value);
                }
            }
        }
        
        PaymentInfo {
            uri,
            payment_type,
            is_reusable,
            parameters,
            original_instructions: instructions.into(),
        }
    }
}