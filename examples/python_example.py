"""
Basic example of using the BIP-353 Python bindings.

Usage:
  python python_example.py <bitcoin-address>
  
Example:
  python python_example.py ₿user@example.com
"""

import sys
from bip353 import PyResolver

def main():
    # Get the address from command line arguments
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <bitcoin-address>")
        print(f"Example: {sys.argv[0]} ₿user@example.com")
        return 1
    
    address = sys.argv[1]
    
    # Create a resolver
    try:
        resolver = PyResolver()
    except Exception as e:
        print(f"Failed to create resolver: {e}")
        return 1
    
    # Parse the address to show the user and domain parts
    try:
        user, domain = resolver.parse_address(address)
        print("Parsed address:")
        print(f"  User:   {user}")
        print(f"  Domain: {domain}")
        print()
    except Exception as e:
        print(f"Failed to parse address: {e}")
        return 1
    
    # Resolve the address
    print(f"Resolving address {address}...")
    try:
        info = resolver.resolve_address(address)
        print("Resolution successful!")
        print(f"  URI:      {info.uri}")
        print(f"  Type:     {info.payment_type}")
        print(f"  Reusable: {info.is_reusable}")
        
        if info.parameters:
            print("  Parameters:")
            for key, value in info.parameters.items():
                print(f"    {key}: {value}")
    except Exception as e:
        print(f"Failed to resolve address: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())