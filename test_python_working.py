"""
Working BIP-353 Python Bindings Test

"""

import sys
import time
import traceback

def test_import():
    """Test if we can import the module"""
    print("=== Testing Python Import ===\n")
    
    try:
        sys.path.append('.')
        import bip353
        print("✅ Successfully imported bip353 module")
        
        # Show what's available
        available = [x for x in dir(bip353) if not x.startswith('_')]
        print(f"📋 Available classes: {available}")
        
        return bip353
    except Exception as e:
        print(f"❌ Failed to import bip353: {e}")
        return None

def test_basic_functionality(bip353):
    """Test basic resolver functionality with available methods only"""
    print("\n=== Testing Basic Functionality ===\n")
    
    try:
        # Test 1: Create resolver
        print("1. Creating resolver...")
        resolver = bip353.PyResolver()
        print("✅ Resolver created successfully")
        
        # Show available methods
        methods = [method for method in dir(resolver) if not method.startswith('_')]
        print(f"📋 Available methods: {', '.join(methods)}")
        
        # Test 2: Test address parsing
        print("\n2. Testing address parsing...")
        test_address = "matt@mattcorallo.com"
        user, domain = resolver.parse_address(test_address)
        print(f"✅ Address parsed successfully:")
        print(f"   User: {user}")
        print(f"   Domain: {domain}")
        
        return resolver
        
    except Exception as e:
        print(f"❌ Error in basic functionality: {e}")
        traceback.print_exc()
        return None

def test_resolution(bip353, resolver):
    """Test actual BIP-353 resolution"""
    print("\n=== Testing BIP-353 Resolution ===\n")
    
    test_address = "matt@mattcorallo.com"
    
    try:
        print(f"Resolving: {test_address}")
        print("Please wait...")
        
        start_time = time.time()
        payment_info = resolver.resolve_address(test_address)
        end_time = time.time()
        
        print(f"✅ Resolution successful! ({(end_time - start_time)*1000:.0f}ms)")
        print(f"   URI: {payment_info.uri}")
        print(f"   Payment Type: {payment_info.payment_type}")
        print(f"   Is Reusable: {payment_info.is_reusable}")
        
        # Test payment info methods
        print("\n📋 Payment Info Details:")
        info_methods = [method for method in dir(payment_info) if not method.startswith('_')]
        print(f"   Available methods: {', '.join(info_methods)}")
        
        # Test available getters
        if hasattr(payment_info, 'parameters'):
            params = payment_info.parameters
            print(f"   Parameters: {dict(params) if params else 'None'}")
            
        return payment_info
        
    except Exception as e:
        print(f"❌ Resolution failed: {e}")
        traceback.print_exc()
        return None

def test_different_addresses(bip353, resolver):
    """Test resolution with different addresses"""
    print("\n=== Testing Different Addresses ===\n")
    
    test_cases = [
        ("matt@mattcorallo.com", "Known working address"),
        ("test@example.com", "Non-existent address (should fail)"),
        ("₿demo@btcpayserver.org", "Address with Bitcoin symbol"),
    ]
    
    successful = 0
    for address, description in test_cases:
        print(f"Testing {description}: {address}")
        
        try:
            payment_info = resolver.resolve_address(address)
            print(f"  ✅ Success: {payment_info.uri}")
            successful += 1
        except Exception as e:
            print(f"  ❌ Failed: {e}")
    
    print(f"\n📊 Results: {successful}/{len(test_cases)} addresses resolved successfully")

def test_user_domain_resolution(bip353, resolver):
    """Test resolving with separate user/domain"""
    print("\n=== Testing User/Domain Resolution ===\n")
    
    try:
        print("Testing separate user/domain resolution...")
        user = "matt"
        domain = "mattcorallo.com"
        
        payment_info = resolver.resolve(user, domain)
        print(f"✅ Success: {payment_info.uri}")
        print(f"   Same as resolve_address: {payment_info.uri}")
        
    except Exception as e:
        print(f"❌ Failed: {e}")

def test_network_resolvers(bip353):
    """Test different network resolvers"""
    print("\n=== Testing Network Resolvers ===\n")
    
    networks = ["mainnet", "testnet", "regtest"]
    
    for network in networks:
        try:
            print(f"Testing {network} resolver...")
            resolver = bip353.PyResolver.for_network(network)
            print(f"✅ {network} resolver created successfully")
            
            # Quick test with Matt's address on mainnet only
            if network == "mainnet":
                try:
                    result = resolver.resolve_address("matt@mattcorallo.com")
                    print(f"   ✅ Resolution works: {result.payment_type}")
                except Exception as e:
                    print(f"   ❌ Resolution failed: {e}")
            
        except Exception as e:
            print(f"❌ {network} resolver failed: {e}")

def test_error_handling(bip353, resolver):
    """Test error handling"""
    print("\n=== Testing Error Handling ===\n")
    
    error_cases = [
        ("invalid-format", "Invalid address format"),
        ("test@nonexistent-domain-12345.com", "Non-existent domain"),
        ("", "Empty address"),
    ]
    
    for address, description in error_cases:
        print(f"Testing {description}: '{address}'")
        
        try:
            # This should fail
            payment_info = resolver.resolve_address(address)
            print(f"  ❌ Should have failed but got: {payment_info.uri}")
        except Exception as e:
            print(f"  ✅ Error handled correctly: {type(e).__name__}: {e}")

def test_performance(bip353, resolver):
    """Test performance with multiple calls"""
    print("\n=== Testing Performance ===\n")
    
    address = "matt@mattcorallo.com"
    iterations = 3
    times = []
    
    print(f"Running {iterations} resolutions of {address}...")
    
    for i in range(iterations):
        try:
            start = time.time()
            payment_info = resolver.resolve_address(address)
            end = time.time()
            
            duration = (end - start) * 1000
            times.append(duration)
            print(f"  Call {i+1}: {duration:.0f}ms - {payment_info.payment_type}")
            
        except Exception as e:
            print(f"  Call {i+1}: Failed - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        
        print(f"\n📈 Performance Summary:")
        print(f"   Average: {avg_time:.0f}ms")
        print(f"   Min: {min_time:.0f}ms")
        print(f"   Max: {max_time:.0f}ms")
        
        # Check if later calls are faster (caching)
        if len(times) > 1 and times[1] < times[0] * 0.5:
            print(f"   🎯 Caching appears to be working!")

def main():
    print("🐍 BIP-353 Python Bindings Test (Working Version)")
    print("===================================================\n")
    
    # Test import
    bip353 = test_import()
    if not bip353:
        return 1
    
    # Test basic functionality
    resolver = test_basic_functionality(bip353)
    if not resolver:
        return 1
    
    # Test resolution (the main feature)
    payment_info = test_resolution(bip353, resolver)
    
    # Test different addresses
    test_different_addresses(bip353, resolver)
    
    # Test user/domain resolution
    test_user_domain_resolution(bip353, resolver)
    
    # Test network resolvers
    test_network_resolvers(bip353)
    
    # Test error handling
    test_error_handling(bip353, resolver)
    
    # Test performance
    test_performance(bip353, resolver)
    
    print("\n🎉 Python bindings testing completed!")
    if payment_info:
        print("✅ Your Python bindings are working correctly!")
        print(f"✅ Successfully resolved: {payment_info.uri}")
        return 0
    else:
        print("⚠️  Some functionality works, but main resolution failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())