#!/bin/bash

# Release script for oat CLI tool
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if we have uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    print_warning "You have uncommitted changes. Please commit or stash them before releasing."
    git status --short
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Get current version from Cargo.toml
current_version=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
print_info "Current version: $current_version"

# Parse version components
IFS='.' read -ra VERSION_PARTS <<< "$current_version"
major=${VERSION_PARTS[0]}
minor=${VERSION_PARTS[1]}
patch=${VERSION_PARTS[2]}

# Determine new version based on argument
case "${1:-patch}" in
    "major")
        new_major=$((major + 1))
        new_minor=0
        new_patch=0
        ;;
    "minor")
        new_major=$major
        new_minor=$((minor + 1))
        new_patch=0
        ;;
    "patch")
        new_major=$major
        new_minor=$minor
        new_patch=$((patch + 1))
        ;;
    *)
        print_error "Invalid version type. Use: patch, minor, or major"
        exit 1
        ;;
esac

new_version="${new_major}.${new_minor}.${new_patch}"
print_info "New version: $new_version"

# Confirm the release
read -p "Release version $new_version? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Release cancelled."
    exit 0
fi

# Update version in Cargo.toml
print_info "Updating version in Cargo.toml..."
sed -i.bak "s/version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
print_info "Updating Cargo.lock..."
cargo check > /dev/null 2>&1

# Commit the version change
print_info "Committing version change..."
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $new_version"

# Create and push tag
print_info "Creating and pushing tag v$new_version..."
git tag "v$new_version"
git push origin main
git push origin "v$new_version"

print_info "✅ Release v$new_version initiated!"
print_info "GitHub Actions will now build and publish the release."
print_info "Check the Actions tab in your GitHub repository for progress."

# Optional: Open GitHub releases page
if command -v open &> /dev/null; then
    read -p "Open GitHub releases page? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "https://github.com/Prixix/oat/releases"
    fi
fi 