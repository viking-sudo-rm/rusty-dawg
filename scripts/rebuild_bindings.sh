#!/usr/bin/env sh

# Parse command-line arguments
NO_INSTALL=false
BUILD_SDIST=false
ARCH_TARGET=""
while [ $# -gt 0 ]; do
    case "$1" in
        --no-install)
            NO_INSTALL=true
            shift
            ;;
        --sdist)
            BUILD_SDIST=true
            shift
            ;;
        --target)
            if [ -n "$2" ]; then
                ARCH_TARGET="$2"
                shift 2
            else
                echo "Error: --target requires an argument" >&2
                exit 1
            fi
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
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

# Architecture target
if [ -n "$ARCH_TARGET" ]; then
    ARCH_TARGET="--target $ARCH_TARGET"
fi

# Add the version to Cargo.toml in the [package] section
sed -i.bak "/^\[package\]/a version = \"$VERSION\"" Cargo.toml

# Flush the wheel cache
rm -rf target/wheels

# Build the bindings; we operate slightly differently depending on whether we're building an sdist or not
# and whether we're installing the wheel or not
if [ "$NO_INSTALL" = false ]; then
    set -ex
    python -m maturin build --release
    python -m pip install target/wheels/* --ignore-installed
    set +x
fi

if [ "$BUILD_SDIST" = true ]; then
    set -ex
    python -m maturin sdist --out "$DIST_DIR"
    set +x
fi

if [ "$NO_INSTALL" = true ]; then
    set -ex
    python -m maturin build --release --out "$DIST_DIR" $ARCH_TARGET
    set +x
fi

# Ensure Cargo.toml is always restored, even if an error occurred
mv Cargo.toml.bak Cargo.toml

# Return to the original directory
cd "$CURRENT_DIR"
