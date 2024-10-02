#!/usr/bin/env sh

# Parse command-line arguments
NO_INSTALL=false
BUILD_SDIST=false
for arg in "$@"
do
    case $arg in
        --no-install)
        NO_INSTALL=true
        shift # Remove --no-install from processing
        ;;
        --sdist)
        BUILD_SDIST=true
        shift # Remove --sdist from processing
        ;;
        *)
        # Unknown option
        ;;
    esac
done

# Get the absolute path of the directory containing this script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# location where this script is being run from
CURRENT_DIR="$(pwd)"

# distribution directory
DIST_DIR="$SCRIPT_DIR/../dist"

# Read the version from Cargo.toml
CARGO_TOML="$SCRIPT_DIR/../Cargo.toml"
VERSION=$(awk -F '"' '/^\[package\]/{f=1} f==1 && /^version/{print $2; exit}' "$CARGO_TOML")

if [ -z "$VERSION" ]; then
    echo "Error: Could not find version in Cargo.toml"
    exit 1
fi
echo "Detected version: $VERSION"

# Moving to the bindings/python directory
cd "$SCRIPT_DIR/../bindings/python"

# Create a backup of Cargo.toml
cp Cargo.toml Cargo.toml.bak

# Wrap the operations in a try-catch-like block
{
    # Add the version to Cargo.toml in the [package] section
    sed -i.bak "/^\[package\]/a version = \"$VERSION\"" Cargo.toml

    # Flush the wheel cache
    rm -rf target/wheels

    # Build the bindings; we operate slightly differently depending on whether we're building an sdist or not
    # and whether we're installing the wheel or not
    if [ "$NO_INSTALL" = false ]; then
        python -m maturin build --release
        python -m pip install target/wheels/* --ignore-installed
    fi

    if [ "$BUILD_SDIST" = true ]; then
        python -m maturin sdist --out "$DIST_DIR"
    fi

    if [ "$NO_INSTALL" = true ]; then
        python -m maturin build --release --out "$DIST_DIR"
    fi
} || {
    echo "An error occurred during the build process"
    ERROR_OCCURRED=true
}

# Ensure Cargo.toml is always restored, even if an error occurred
mv Cargo.toml.bak Cargo.toml

# Return to the original directory
cd "$CURRENT_DIR"

# Exit with error code 1 if an error occurred
if [ "$ERROR_OCCURRED" = true ]; then
    exit 1
fi
