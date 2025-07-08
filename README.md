# bip353-rs

[![crates.io](https://img.shields.io/crates/v/bip353-integrations.svg)](https://crates.io/crates/bip353-integrations)
[![Documentation](https://docs.rs/bip353-integrations/badge.svg)](https://docs.rs/bip353-integrations)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/bip353-integrations.svg)](./LICENSE)

**BIP-353 Integrations** is a high-level integration layer for BIP-353 (DNS Payment Instructions) that provides convenient interfaces for Bitcoin Core and HWI integration. This library wraps the underlying [`bitcoin-payment-instructions`](https://crates.io/crates/bitcoin-payment-instructions) and [`dnssec-prover`](https://crates.io/crates/dnssec-prover) crates by Matt Corallo, providing a simplified API, FFI bindings, and Python bindings.

## Overview

BIP-353 enables human-readable Bitcoin addresses in the format `user@domain` (or `₿user@domain`), similar to email addresses. This makes Bitcoin payments significantly more user-friendly while maintaining strong security guarantees through DNSSEC.

This library provides an ergonomic way to integrate BIP-353 into various Bitcoin applications:

- **Bitcoin Core Integration**: FFI bindings for C/C++ code
- **HWI Integration**: Python bindings for hardware wallet interactions
- **Simplified API**: High-level interfaces for common operations
- **Security First**: Built on production-ready, security-focused crates

## Installation

### Rust

Add this to your `Cargo.toml`:

```toml
[dependencies]
bip353-integrations = "0.1.0"
```

### C/C++ (FFI)

Enable the `ffi` feature:

```toml
[dependencies]
bip353-integrations = { version = "0.1.0", features = ["ffi"] }
```

Then build a static or dynamic library:

```bash
cargo build --release --features ffi
```

### Python

Enable the `python` feature:

```toml
[dependencies]
bip353-integrations = { version = "0.1.0", features = ["python"] }
```

Then build the Python extension:

```bash
cargo build --release --features python
```

## Usage

### Rust

```rust
use bip353::{Bip353Resolver, ResolverConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a resolver with default configuration (mainnet)
    let resolver = Bip353Resolver::new()?;
    
    // Or with custom configuration
    let config = ResolverConfig::testnet()
        .with_timeout(std::time::Duration::from_secs(10));
    let testnet_resolver = Bip353Resolver::with_config(config)?;
    
    // Parse a BIP-353 address
    let (user, domain) = bip353::parse_address("₿alice@example.com")?;
    println!("User: {}, Domain: {}", user, domain);
    
    // Resolve a BIP-353 address
    match resolver.resolve_address("₿alice@example.com").await {
        Ok(info) => {
            println!("URI: {}", info.uri);
            println!("Type: {:?}", info.payment_type);
            println!("Reusable: {}", info.is_reusable);
        },
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}
```

### C/C++ (FFI)

```c
#include <stdio.h>
#include "bip353.h"

int main() {
    // Create a resolver
    BIP353_ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        printf("Failed to create resolver\n");
        return 1;
    }
    
    // Resolve an address
    Bip353Result* result = bip353_resolve_address(resolver, "₿alice@example.com");
    if (!result) {
        printf("Failed to resolve address\n");
        bip353_resolver_free(resolver);
        return 1;
    }
    
    if (result->success) {
        printf("URI: %s\n", result->uri);
        printf("Type: %s\n", result->payment_type);
        printf("Reusable: %s\n", result->is_reusable ? "true" : "false");
    } else {
        printf("Error: %s\n", result->error);
    }
    
    // Free resources
    bip353_result_free(result);
    bip353_resolver_free(resolver);
    
    return 0;
}
```

### Python

```python
from bip353 import PyResolver, PyPaymentInfo

def main():
    # Create a resolver
    resolver = PyResolver()
    
    # Or for testnet
    testnet_resolver = PyResolver.for_network("testnet")
    
    # Parse an address
    user, domain = resolver.parse_address("₿alice@example.com")
    print(f"User: {user}, Domain: {domain}")
    
    # Resolve an address
    try:
        info = resolver.resolve_address("₿alice@example.com")
        print(f"URI: {info.uri}")
        print(f"Type: {info.payment_type}")
        print(f"Reusable: {info.is_reusable}")
        print(f"Parameters: {info.parameters}")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
```

## Example: Bitcoin Core Integration

Here's how you might implement a new RPC call in Bitcoin Core using the FFI bindings:

```cpp
static UniValue resolvebitcoinaddress(const JSONRPCRequest& request)
{
    RPCHelpMan{"resolvebitcoinaddress",
        "Resolves a human-readable Bitcoin address (₿user@domain).",
        {
            {"address", RPCArg::Type::STR, RPCArg::Optional::NO, "The human-readable Bitcoin address"}
        },
        RPCResult{
            RPCResult::Type::OBJ, "", "",
            {
                {RPCResult::Type::STR, "uri", "The BIP-21 URI"},
                {RPCResult::Type::STR, "type", "The payment type"},
                {RPCResult::Type::BOOL, "is_reusable", "Whether the address is reusable"}
            }
        },
        RPCExamples{
            HelpExampleCli("resolvebitcoinaddress", "\"₿alice@example.com\"")
        },
    }.Check(request);

    std::string address = request.params[0].get_str();
    
    // Create resolver (could be cached globally)
    BIP353_ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        throw JSONRPCError(RPC_INTERNAL_ERROR, "Failed to create BIP-353 resolver");
    }
    
    // Resolve the address
    Bip353Result* result = bip353_resolve_address(resolver, address.c_str());
    
    // Free the resolver
    bip353_resolver_free(resolver);
    
    if (!result) {
        throw JSONRPCError(RPC_INTERNAL_ERROR, "Failed to resolve address");
    }
    
    UniValue response(UniValue::VOBJ);
    
    if (result->success) {
        response.pushKV("uri", std::string(result->uri));
        response.pushKV("type", std::string(result->payment_type));
        response.pushKV("is_reusable", result->is_reusable);
    } else {
        std::string error = result->error ? result->error : "Unknown error";
        bip353_result_free(result);
        throw JSONRPCError(RPC_INTERNAL_ERROR, error);
    }
    
    bip353_result_free(result);
    
    return response;
}
```

## Example: HWI Integration

Here's how you might integrate with HWI using the Python bindings:

```python
from hwi.errors import ActionCanceledError, HWWError
from hwi.device_manager import HardwareDeviceClient
from bip353 import PyResolver

class BIP353Mixin:
    """Mixin for HWI to support BIP-353 resolution."""
    
    def resolve_bitcoin_address(self, address: str):
        """Resolve a human-readable Bitcoin address."""
        try:
            resolver = PyResolver()
            info = resolver.resolve_address(address)
            
            return {
                "uri": info.uri,
                "type": info.payment_type,
                "is_reusable": info.is_reusable,
                "parameters": info.parameters
            }
        except Exception as e:
            raise HWWError(f"Failed to resolve BIP-353 address: {e}")

# Add the mixin to HardwareDeviceClient
HardwareDeviceClient.__bases__ = (BIP353Mixin,) + HardwareDeviceClient.__bases__
```

## Security Considerations

This library prioritizes security in several ways:

1. **DNSSEC Validation**: Enforces DNSSEC validation for DNS lookups
2. **Proper Error Handling**: Validates inputs and handles errors thoroughly
3. **Memory Safety**: Uses Rust's safety guarantees and careful FFI design
4. **No Unnecessary Dependencies**: Minimizes the dependency tree

## License

This library is licensed under:

 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

## Acknowledgments

This library is built on top of Matt Corallo's [`bitcoin-payment-instructions`](https://crates.io/crates/bitcoin-payment-instructions) and [`dnssec-prover`](https://crates.io/crates/dnssec-prover) crates, which provide the core functionality for BIP-353.
