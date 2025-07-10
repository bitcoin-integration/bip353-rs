use crate::{PaymentInfo, Bip353Error};
use std::collections::HashMap;

/// Metadata that wallets can use for BIP-353 payments
#[derive(Debug, Clone)]
pub struct WalletPaymentInfo {
    /// The resolved payment info
    pub payment_info: PaymentInfo,
    
    /// Human-readable name for display
    pub display_name: String,
    
    /// Metadata for wallet integration
    pub metadata: WalletMetadata,
}

/// Metadata that different wallet types might need
#[derive(Debug, Clone)]
pub struct WalletMetadata {
    /// Original BIP-353 address (for labeling)
    pub original_address: String,
    
    /// DNSSEC proof (for hardware wallet verification)
    pub dnssec_proof: Option<Vec<u8>>,
    
    /// Suggested label for the transaction
    pub suggested_label: String,
    
    /// Additional parameters for specific wallet types
    pub wallet_specific: HashMap<String, String>,
}

/// Simple wallet integration helpers
/// Please note that this doesn't try to implement wallet-specific logic, just provides helpers
pub struct WalletIntegrationHelper;

impl WalletIntegrationHelper {
    /// Prepare payment info for wallet integration
    pub fn prepare_for_wallet(
        payment_info: PaymentInfo,
        original_address: &str,
        wallet_type: WalletType,
    ) -> WalletPaymentInfo {
        let mut metadata = WalletMetadata {
            original_address: original_address.to_string(),
            dnssec_proof: None, 
            suggested_label: format!("BIP-353: {}", original_address),
            wallet_specific: HashMap::new(),
        };
        
        // Add wallet-specific metadata without implementing wallet logic
        match wallet_type {
            WalletType::Sparrow => {
                metadata.wallet_specific.insert(
                    "sparrow_memo".to_string(),
                    format!("Paid to {}", original_address),
                );
            },
            WalletType::Electrum => {
                metadata.wallet_specific.insert(
                    "electrum_description".to_string(),
                    format!("BIP-353 payment to {}", original_address),
                );
            },
            WalletType::BitcoinCore => {
                metadata.wallet_specific.insert(
                    "core_comment".to_string(),
                    format!("BIP-353: {}", original_address),
                );
            },
            WalletType::BDK => {
                // this would not need special metadata
            },
        }
        
        WalletPaymentInfo {
            payment_info,
            display_name: Self::create_display_name(original_address),
            metadata,
        }
    }
    
    /// Create a user-friendly display name
    fn create_display_name(address: &str) -> String {
        // Remove Bitcoin prefix if present
        let addr = address.strip_prefix("₿").unwrap_or(address);
        format!("₿{}", addr)
    }
    
    /// Extract parameters for BIP-21 URI construction
    pub fn extract_bip21_params(wallet_info: &WalletPaymentInfo) -> HashMap<String, String> {
        let mut params = wallet_info.payment_info.parameters.clone();
        
        // Add BIP-353 specific parameters
        params.insert("label".to_string(), wallet_info.metadata.suggested_label.clone());
        params.insert("message".to_string(), format!("Payment to {}", wallet_info.metadata.original_address));
        
        params
    }
}

/// Example wallet types (please add to this list as you wish!)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletType {
    Sparrow,
    Electrum,
    BitcoinCore,
    BDK,
}

impl std::fmt::Display for WalletType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletType::Sparrow => write!(f, "sparrow"),
            WalletType::Electrum => write!(f, "electrum"),
            WalletType::BitcoinCore => write!(f, "bitcoin-core"),
            WalletType::BDK => write!(f, "bdk"),
        }
    }
}

/// Trait for wallet-specific integrations
/// Implementors provide wallet-specific logic, we provide the BIP-353 resolution
pub trait WalletIntegration {
    type Error: std::error::Error + Send + Sync + 'static;
    type TransactionOutput;
    
    /// Create a transaction with a resolved BIP-353 payment info
    /// The actual transaction creation is wallet-specific
    async fn create_bip353_transaction(
        &self,
        wallet_info: WalletPaymentInfo,
        amount: bitcoin::Amount,
    ) -> Result<Self::TransactionOutput, Self::Error>;
}

// Example trait implementation would be provided by wallet developers:
/*
impl WalletIntegration for SparrowWallet {
    type Error = SparrowError;
    type TransactionOutput = PSBT;
    
    async fn create_bip353_transaction(
        &self,
        wallet_info: WalletPaymentInfo,
        amount: bitcoin::Amount,
    ) -> Result<PSBT, SparrowError> {
        todo!()
    }
}
*/