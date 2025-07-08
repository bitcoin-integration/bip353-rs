//! Error types for BIP-353 operations

use thiserror::Error;

/// Main error type for BIP-353 operations
#[derive(Error, Debug)]
pub enum Bip353Error {
    /// DNS resolution or DNSSEC validation error
    #[error("DNS error: {0}")]
    DnsError(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid record or parsing error
    #[error("Invalid record: {0}")]
    InvalidRecord(String),

    /// DNSSEC validation failed
    #[error("DNSSEC error: {0}")]
    DnssecError(String),

    /// Generic error from the underlying implementation
    #[error("Implementation error: {0}")]
    ImplError(String),

    /// Network or I/O error
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<bitcoin_payment_instructions::ParseError> for Bip353Error {
    fn from(err: bitcoin_payment_instructions::ParseError) -> Self {
        match err {
            bitcoin_payment_instructions::ParseError::InvalidOnChain(_) => {
                Bip353Error::InvalidRecord("Invalid on-chain address format".into())
            },
            bitcoin_payment_instructions::ParseError::InvalidBolt11(_) => {
                Bip353Error::InvalidRecord("Invalid Lightning invoice format".into())
            },
            bitcoin_payment_instructions::ParseError::InvalidBolt12(_) => {
                Bip353Error::InvalidRecord("Invalid Lightning offer format".into())
            },
            bitcoin_payment_instructions::ParseError::WrongNetwork => {
                Bip353Error::InvalidRecord("Payment instruction for wrong network".into())
            },
            bitcoin_payment_instructions::ParseError::InconsistentInstructions(msg) => {
                Bip353Error::InvalidRecord(format!("Inconsistent payment instructions: {}", msg))
            },
            bitcoin_payment_instructions::ParseError::InvalidInstructions(msg) => {
                Bip353Error::InvalidRecord(format!("Invalid payment instructions: {}", msg))
            },
            bitcoin_payment_instructions::ParseError::UnknownPaymentInstructions => {
                Bip353Error::InvalidRecord("Unknown payment instruction format".into())
            },
            bitcoin_payment_instructions::ParseError::UnknownRequiredParameter => {
                Bip353Error::InvalidRecord("Unknown required parameter in payment URI".into())
            },
            bitcoin_payment_instructions::ParseError::HrnResolutionError(msg) => {
                Bip353Error::DnsError(msg.to_string())
            },
            bitcoin_payment_instructions::ParseError::InstructionsExpired => {
                Bip353Error::InvalidRecord("Payment instructions have expired".into())
            },
        }
    }
}

// Convert HrnResolutionError to our error type
impl From<&'static str> for Bip353Error {
    fn from(err: &'static str) -> Self {
        Bip353Error::DnsError(err.to_string())
    }
}

// For FFI error handling
#[cfg(feature = "ffi")]
impl Bip353Error {
    pub(crate) fn to_string_representation(&self) -> String {
        self.to_string()
    }
}