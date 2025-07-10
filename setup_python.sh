
# creating setup_python.sh for CLi

echo "🐍 Setting up Python bindings..."

# Build the Python extension
echo "Building Python extension..."
cargo build --features python --release

# Find the generated .so file
RUST_SO=$(find target/release -name "*.so" -type f | head -1)

if [ -z "$RUST_SO" ]; then
    echo "❌ No .so file found. Trying debug build..."
    cargo build --features python
    RUST_SO=$(find target/debug -name "*.so" -type f | head -1)
fi

if [ -z "$RUST_SO" ]; then
    echo "❌ Still no .so file found. Let's check what was built:"
    echo "Files in target/debug:"
    ls -la target/debug/
    echo "Files in target/release:"
    ls -la target/release/ 2>/dev/null || echo "No release directory"
    exit 1
fi

echo "✅ Found Rust library: $RUST_SO"

# Create a Python-importable module
# The .so file needs to be named correctly for Python to import it
if [[ "$RUST_SO" == *"libbip353"* ]]; then
    # Copy and rename for Python import
    PYTHON_SO="bip353.so"
    cp "$RUST_SO" "$PYTHON_SO"
    echo "✅ Created Python module: $PYTHON_SO"
    
    # Test the import
    echo "Testing Python import..."
    python3 -c "import bip353; print('✅ Import successful!')" 2>/dev/null || {
        echo "❌ Import failed. Let's check the file:"
        ls -la bip353.so
        file bip353.so
        echo "Python path:"
        python3 -c "import sys; print(sys.path)"
    }
else
    echo "❌ Unexpected .so filename: $RUST_SO"
    exit 1
fi