// test_ffi.c

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Copy the basic FFI declarations (since we don't have the full header yet)
typedef struct ResolverPtr ResolverPtr;
typedef struct Bip353Result {
    int success;
    char* uri;
    char* payment_type;
    int is_reusable;
    char* error;
} Bip353Result;

// Function declarations - these should match your FFI
extern ResolverPtr* bip353_resolver_create(void);
extern void bip353_resolver_free(ResolverPtr* ptr);
extern Bip353Result* bip353_resolve_address(const ResolverPtr* ptr, const char* address);
extern void bip353_result_free(Bip353Result* ptr);
extern int bip353_parse_address(const char* address, char** user_out, char** domain_out);
extern void bip353_string_free(char* ptr);

void test_basic_ffi() {
    printf("=== Testing Basic FFI Functionality ===\n\n");
    
    // Test 1: Create resolver
    printf("1. Creating resolver...\n");
    ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        printf("❌ Failed to create resolver\n");
        return;
    }
    printf("✅ Resolver created successfully\n\n");
    
    // Test 2: Parse address
    printf("2. Testing address parsing...\n");
    char* user = NULL;
    char* domain = NULL;
    const char* test_address = "matt@mattcorallo.com";
    
    if (bip353_parse_address(test_address, &user, &domain)) {
        printf("✅ Address parsed successfully:\n");
        printf("   User: %s\n", user);
        printf("   Domain: %s\n", domain);
        
        // Free the strings
        bip353_string_free(user);
        bip353_string_free(domain);
    } else {
        printf("❌ Failed to parse address\n");
    }
    printf("\n");
    
    // Test 3: Resolve address (this is the real test!)
    printf("3. Testing BIP-353 resolution...\n");
    printf("   Resolving: %s\n", test_address);
    printf("   Please wait...\n");
    
    Bip353Result* result = bip353_resolve_address(resolver, test_address);
    if (!result) {
        printf("❌ Failed to get result\n");
        bip353_resolver_free(resolver);
        return;
    }
    
    if (result->success) {
        printf("✅ Resolution successful!\n");
        printf("   URI: %s\n", result->uri);
        printf("   Payment Type: %s\n", result->payment_type);
        printf("   Reusable: %s\n", result->is_reusable ? "Yes" : "No");
    } else {
        printf("❌ Resolution failed: %s\n", result->error ? result->error : "Unknown error");
    }
    
    // Test 4: Free resources
    printf("\n4. Cleaning up...\n");
    bip353_result_free(result);
    bip353_resolver_free(resolver);
    printf("✅ Resources freed successfully\n");
}

void test_error_handling() {
    printf("\n=== Testing Error Handling ===\n\n");
    
    ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        printf("❌ Failed to create resolver\n");
        return;
    }
    
    // Test with invalid address
    printf("1. Testing invalid address...\n");
    Bip353Result* result = bip353_resolve_address(resolver, "invalid-address");
    if (!result) {
        printf("❌ Failed to get result\n");
    } else {
        if (!result->success) {
            printf("✅ Error handled correctly: %s\n", result->error ? result->error : "Unknown error");
        } else {
            printf("❌ Should have failed but didn't\n");
        }
        bip353_result_free(result);
    }
    
    // Test with non-existent domain
    printf("\n2. Testing non-existent domain...\n");
    result = bip353_resolve_address(resolver, "test@nonexistent-domain-12345.com");
    if (!result) {
        printf("❌ Failed to get result\n");
    } else {
        if (!result->success) {
            printf("✅ DNS error handled correctly: %s\n", result->error ? result->error : "Unknown error");
        } else {
            printf("❌ Should have failed but didn't\n");
        }
        bip353_result_free(result);
    }
    
    bip353_resolver_free(resolver);
}

void test_multiple_resolutions() {
    printf("\n=== Testing Multiple Resolutions ===\n\n");
    
    ResolverPtr* resolver = bip353_resolver_create();
    if (!resolver) {
        printf("❌ Failed to create resolver\n");
        return;
    }
    
    const char* addresses[] = {
        "matt@mattcorallo.com",
        "test@example.com",
        "₿demo@btcpayserver.org"
    };
    
    int num_addresses = sizeof(addresses) / sizeof(addresses[0]);
    int successful = 0;
    
    for (int i = 0; i < num_addresses; i++) {
        printf("%d. Testing: %s\n", i + 1, addresses[i]);
        
        Bip353Result* result = bip353_resolve_address(resolver, addresses[i]);
        if (result) {
            if (result->success) {
                printf("   ✅ Success: %s\n", result->uri);
                successful++;
            } else {
                printf("   ❌ Failed: %s\n", result->error ? result->error : "Unknown error");
            }
            bip353_result_free(result);
        } else {
            printf("   ❌ Failed to get result\n");
        }
    }
    
    printf("\nSummary: %d/%d addresses resolved successfully\n", successful, num_addresses);
    bip353_resolver_free(resolver);
}

int main() {
    printf("🔗 BIP-353 FFI Integration Test\n");
    printf("================================\n\n");
    
    test_basic_ffi();
    test_error_handling();
    test_multiple_resolutions();
    
    printf("\n🎉 FFI testing completed!\n");
    printf("If you see this message, the FFI bindings are working correctly.\n");
    
    return 0;
}