#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    print_error "GitHub CLI (gh) is not installed. Please install it first."
    echo "Visit: https://cli.github.com/"
    exit 1
fi

# Check if jq is installed (for JSON parsing)
if ! command -v jq &> /dev/null; then
    print_error "jq is not installed. Please install it first."
    echo "  Ubuntu/Debian: sudo apt-get install jq"
    echo "  Fedora: sudo dnf install jq"
    echo "  macOS: brew install jq"
    echo "  Arch: sudo pacman -S jq"
    exit 1
fi

# Get current version from package.json
if [ ! -f "package.json" ]; then
    print_error "package.json not found. Are you in the project root?"
    exit 1
fi

CURRENT_VERSION=$(jq -r '.version' package.json)
print_info "Current version: ${GREEN}${CURRENT_VERSION}${NC}"

# Ask for new version
echo ""
read -p "Enter new version (e.g., 0.2.0): " NEW_VERSION

# Validate version format (basic semver check)
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Please use semantic versioning (e.g., 0.2.0)"
    exit 1
fi

echo ""
print_warning "This will update the version to ${GREEN}${NEW_VERSION}${NC} in:"
echo "  • package.json"
echo "  • PKGBUILD"
echo "  • src-tauri/Cargo.toml"
echo "  • src-tauri/tauri.conf.json"
echo ""
read -p "Continue? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Aborted."
    exit 0
fi

echo ""
print_info "Updating versions..."

# 1. Update package.json
print_info "Updating package.json..."
jq --arg version "$NEW_VERSION" '.version = $version' package.json > package.json.tmp && mv package.json.tmp package.json
print_success "Updated package.json"

# 2. Update PKGBUILD
if [ -f "PKGBUILD" ]; then
    print_info "Updating PKGBUILD..."
    sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" PKGBUILD
    makepkg --printsrcinfo > .SRCINFO
    print_success "Updated PKGBUILD"
else
    print_warning "PKGBUILD not found, skipping..."
fi

# 3. Update src-tauri/Cargo.toml
if [ -f "src-tauri/Cargo.toml" ]; then
    print_info "Updating src-tauri/Cargo.toml..."
    sed -i "0,/^version = .*/{s/^version = .*/version = \"${NEW_VERSION}\"/}" src-tauri/Cargo.toml
    print_success "Updated src-tauri/Cargo.toml"
else
    print_error "src-tauri/Cargo.toml not found!"
    exit 1
fi

# 4. Update src-tauri/tauri.conf.json
if [ -f "src-tauri/tauri.conf.json" ]; then
    print_info "Updating src-tauri/tauri.conf.json..."
    jq --arg version "$NEW_VERSION" '.version = $version' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
    print_success "Updated src-tauri/tauri.conf.json"
else
    print_error "src-tauri/tauri.conf.json not found!"
    exit 1
fi

echo ""
print_success "All versions updated to ${GREEN}${NEW_VERSION}${NC}"

# Commit changes
echo ""
read -p "Commit these changes? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Committing changes..."
    git add package.json PKGBUILD src-tauri/Cargo.toml src-tauri/tauri.conf.json 2>/dev/null || true
    git commit -m "chore: bump version to ${NEW_VERSION}"
    print_success "Changes committed"
    
    echo ""
    read -p "Push to remote? (y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Pushing to remote..."
        git push
        print_success "Pushed to remote"
    fi
fi

# Trigger release workflow
echo ""
read -p "Trigger release workflow for v${NEW_VERSION}? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Triggering release workflow..."
    
    if gh workflow run publish.yml -f version="v${NEW_VERSION}"; then
        print_success "Release workflow triggered for v${NEW_VERSION}"
        echo ""
        print_info "Check workflow status:"
        echo "  gh run list --workflow=publish.yml"
        echo "  gh run watch"
        echo ""
        print_info "Or visit: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/actions/workflows/publish.yml"
    else
        print_error "Failed to trigger workflow. Make sure you have pushed the publish.yml workflow file."
    fi
else
    print_info "You can manually trigger the release later with:"
    echo "  gh workflow run publish.yml -f version=\"v${NEW_VERSION}\""
fi

echo ""
print_success "Done! 🎉"