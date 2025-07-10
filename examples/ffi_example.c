#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Include the C header for the FFI bindings
// this should be generated or provided with the library
typedef struct ResolverPtr ResolverPtr;
typedef struct Bip353Result {
    int success;
    char* uri;
    char* payment_type;
    int is_reusable;
    char* error;
} Bip353Result;

// Function declarations - these match the functions in src/ffi.rs
ResolverPtr* bip353_resolver_create(void);
ResolverPtr* bip353_resolver_create_with_network(const char* network_name);
void bip353_resolver_free(ResolverPtr* ptr);
Bip353Result* bip353_resolve_address(const ResolverPtr* ptr, const char* address);
Bip353Result* bip353_resolve(const ResolverPtr* ptr, const char* user, const char* domain);
void bip353_result_free(Bip353Result* ptr);
int bip353_parse_address(const char* address, char** user_out, char** domain_out);
void bip353_string_free(char* ptr);

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("Usage: %s <bitcoin-address>\n", argv[0]);
        printf("Example: %s ₿user@example.com\n", argv[0]);
        return 1;
    }
    
    const char* address = argv[1];
    
    // Create a resolver
    ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        fprintf(stderr, "Failed to create resolver\n");
        return 1;
    }
    
    // Parse the address to show the user and domain parts
    char* user = NULL;
    char* domain = NULL;
    if (bip353_parse_address(address, &user, &domain)) {
        printf("Parsed address:\n");
        printf("  User:   %s\n", user);
        printf("  Domain: %s\n", domain);
        printf("\n");
        
        // Free the strings
        bip353_string_free(user);
        bip353_string_free(domain);
    } else {
        fprintf(stderr, "Failed to parse address\n");
        bip353_resolver_free(resolver);
        return 1;
    }
    
    // Resolve the address
    printf("Resolving address %s...\n", address);
    Bip353Result* result = bip353_resolve_address(resolver, address);
    if (!result) {
        fprintf(stderr, "Failed to resolve address\n");
        bip353_resolver_free(resolver);
        return 1;
    }
    
    if (result->success) {
        printf("Resolution successful!\n");
        printf("  URI:      %s\n", result->uri);
        printf("  Type:     %s\n", result->payment_type);
        printf("  Reusable: %s\n", result->is_reusable ? "true" : "false");
    } else {
        fprintf(stderr, "Failed to resolve address: %s\n", result->error);
        bip353_result_free(result);
        bip353_resolver_free(resolver);
        return 1;
    }
    
    // Free resources
    bip353_result_free(result);
    bip353_resolver_free(resolver);
    
    return 0;
}