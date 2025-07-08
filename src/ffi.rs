//! FFI bindings for BIP-353
//!
//! These bindings provide a C API for integration with Bitcoin Core.

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Arc;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use crate::{
    Bip353Error,
    Bip353Resolver,
    ResolverConfig,
    PaymentInfo,
};

// Global runtime for async operations
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create Tokio runtime")
    })
}

/// Opaque pointer for the resolver
pub struct ResolverPtr(Arc<Bip353Resolver>);

/// Create a new resolver with default configuration
#[no_mangle]
pub extern "C" fn bip353_resolver_create() -> *mut ResolverPtr {
    match Bip353Resolver::new() {
        Ok(resolver) => {
            let resolver_ptr = Arc::new(resolver);
            let ptr = Box::new(ResolverPtr(resolver_ptr));
            Box::into_raw(ptr)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Create a new resolver with custom configuration
#[no_mangle]
pub extern "C" fn bip353_resolver_create_with_network(network_name: *const c_char) -> *mut ResolverPtr {
    if network_name.is_null() {
        return ptr::null_mut();
    }
    
    let network_str = match unsafe { CStr::from_ptr(network_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let config = match network_str {
        "main" | "mainnet" | "bitcoin" => ResolverConfig::default(),
        "test" | "testnet" => ResolverConfig::testnet(),
        "signet" => ResolverConfig::signet(),
        "regtest" => ResolverConfig::regtest(),
        _ => return ptr::null_mut(),
    };
    
    match Bip353Resolver::with_config(config) {
        Ok(resolver) => {
            let resolver_ptr = Arc::new(resolver);
            let ptr = Box::new(ResolverPtr(resolver_ptr));
            Box::into_raw(ptr)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Free a resolver
#[no_mangle]
pub extern "C" fn bip353_resolver_free(ptr: *mut ResolverPtr) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

/// Result of a BIP-353 resolution
#[repr(C)]
pub struct Bip353Result {
    /// Whether the resolution was successful
    success: bool,
    
    /// The URI (BIP-21)
    uri: *mut c_char,
    
    /// The payment type
    payment_type: *mut c_char,
    
    /// Whether the payment is reusable
    is_reusable: bool,
    
    /// Error message (if any)
    error: *mut c_char,
}

/// Resolve a human-readable Bitcoin address
#[no_mangle]
pub extern "C" fn bip353_resolve_address(
    ptr: *const ResolverPtr,
    address: *const c_char,
) -> *mut Bip353Result {
    if ptr.is_null() || address.is_null() {
        return ptr::null_mut();
    }
    
    let resolver_ptr = unsafe { &*ptr };
    let resolver = &resolver_ptr.0;
    
    let address_str = match unsafe { CStr::from_ptr(address) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let runtime = get_runtime();
    
    // Resolve the address
    let result = runtime.block_on(async {
        resolver.resolve_address(address_str).await
    });
    
    create_result_ptr(result)
}

/// Resolve a human-readable Bitcoin address from user and domain parts
#[no_mangle]
pub extern "C" fn bip353_resolve(
    ptr: *const ResolverPtr,
    user: *const c_char,
    domain: *const c_char,
) -> *mut Bip353Result {
    if ptr.is_null() || user.is_null() || domain.is_null() {
        return ptr::null_mut();
    }
    
    let resolver_ptr = unsafe { &*ptr };
    let resolver = &resolver_ptr.0;
    
    let user_str = match unsafe { CStr::from_ptr(user) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let domain_str = match unsafe { CStr::from_ptr(domain) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let runtime = get_runtime();
    
    // Resolve the address
    let result = runtime.block_on(async {
        resolver.resolve(user_str, domain_str).await
    });
    
    create_result_ptr(result)
}

fn create_result_ptr(result: Result<PaymentInfo, Bip353Error>) -> *mut Bip353Result {
    let result_ptr = Box::new(match result {
        Ok(info) => {
            // Convert to C strings
            let uri_cstring = match CString::new(info.uri) {
                Ok(s) => s,
                Err(_) => return ptr::null_mut(),
            };
            
            let type_str = info.payment_type.to_string();
            let type_cstring = match CString::new(type_str) {
                Ok(s) => s,
                Err(_) => return ptr::null_mut(),
            };
            
            Bip353Result {
                success: true,
                uri: uri_cstring.into_raw(),
                payment_type: type_cstring.into_raw(),
                is_reusable: info.is_reusable,
                error: ptr::null_mut(),
            }
        }
        Err(err) => {
            let error_str = err.to_string_representation();
            let error_cstring = match CString::new(error_str) {
                Ok(s) => s,
                Err(_) => return ptr::null_mut(),
            };
            
            Bip353Result {
                success: false,
                uri: ptr::null_mut(),
                payment_type: ptr::null_mut(),
                is_reusable: false,
                error: error_cstring.into_raw(),
            }
        }
    });
    
    Box::into_raw(result_ptr)
}

/// Free a result
#[no_mangle]
pub extern "C" fn bip353_result_free(ptr: *mut Bip353Result) {
    if !ptr.is_null() {
        unsafe {
            let result = Box::from_raw(ptr);
            
            // Free the strings
            if !result.uri.is_null() {
                let _ = CString::from_raw(result.uri);
            }
            
            if !result.payment_type.is_null() {
                let _ = CString::from_raw(result.payment_type);
            }
            
            if !result.error.is_null() {
                let _ = CString::from_raw(result.error);
            }
        }
    }
}

/// Parse a human-readable Bitcoin address into user and domain parts
#[no_mangle]
pub extern "C" fn bip353_parse_address(
    address: *const c_char,
    user_out: *mut *mut c_char,
    domain_out: *mut *mut c_char,
) -> bool {
    if address.is_null() || user_out.is_null() || domain_out.is_null() {
        return false;
    }
    
    let address_str = match unsafe { CStr::from_ptr(address) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match crate::parse_address(address_str) {
        Ok((user, domain)) => {
            unsafe {
                match CString::new(user) {
                    Ok(user_cstring) => {
                        *user_out = user_cstring.into_raw();
                    }
                    Err(_) => return false,
                }
                
                match CString::new(domain) {
                    Ok(domain_cstring) => {
                        *domain_out = domain_cstring.into_raw();
                    }
                    Err(_) => {
                        // Free the user string if domain allocation fails
                        let _ = CString::from_raw(*user_out);
                        *user_out = ptr::null_mut();
                        return false;
                    }
                }
            }
            
            true
        }
        Err(_) => false,
    }
}

/// Free a string
#[no_mangle]
pub extern "C" fn bip353_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
