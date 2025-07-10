
/**
 * BIP-353 Integrations - C API
 * 
 * This header defines the C API for the BIP-353 DNS Payment Instructions
 * library. It provides functions for resolving human-readable Bitcoin
 * addresses (₿user@domain) to payment instructions.
 */

#ifndef BIP353_H
#define BIP353_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque pointer for the resolver
 */
typedef struct ResolverPtr ResolverPtr;

/**
 * Result of a BIP-353 resolution
 */
typedef struct Bip353Result {
    /** Whether the resolution was successful */
    int success;
    
    /** The URI (BIP-21) */
    char* uri;
    
    /** The payment type */
    char* payment_type;
    
    /** Whether the payment is reusable */
    int is_reusable;
    
    /** Error message (if any) */
    char* error;
} Bip353Result;

/**
 * Create a new resolver with default configuration
 * 
 * @return A pointer to the resolver, or NULL on error
 */
ResolverPtr* bip353_resolver_create(void);

/**
 * Create a new resolver with a specific network
 * 
 * @param network_name The network name ("main", "testnet", "signet", or "regtest")
 * @return A pointer to the resolver, or NULL on error
 */
ResolverPtr* bip353_resolver_create_with_network(const char* network_name);

/**
 * Free a resolver
 * 
 * @param ptr The resolver to free
 */
void bip353_resolver_free(ResolverPtr* ptr);

/**
 * Resolve a human-readable Bitcoin address
 * 
 * @param ptr The resolver
 * @param address The address to resolve (e.g. "₿user@domain")
 * @return A pointer to the result, or NULL on error
 */
Bip353Result* bip353_resolve_address(const ResolverPtr* ptr, const char* address);

/**
 * Resolve a human-readable Bitcoin address from user and domain parts
 * 
 * @param ptr The resolver
 * @param user The user part
 * @param domain The domain part
 * @return A pointer to the result, or NULL on error
 */
Bip353Result* bip353_resolve(const ResolverPtr* ptr, const char* user, const char* domain);

/**
 * Free a result
 * 
 * @param ptr The result to free
 */
void bip353_result_free(Bip353Result* ptr);

/**
 * Parse a human-readable Bitcoin address into user and domain parts
 * 
 * @param address The address to parse
 * @param user_out Pointer to a variable that will receive the user part
 * @param domain_out Pointer to a variable that will receive the domain part
 * @return 1 on success, 0 on error
 */
int bip353_parse_address(const char* address, char** user_out, char** domain_out);

/**
 * Free a string
 * 
 * @param ptr The string to free
 */
void bip353_string_free(char* ptr);

#ifdef __cplusplus
}
#endif

#endif /* BIP353_H */
