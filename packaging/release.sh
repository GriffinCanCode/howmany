#!/bin/bash

# Release script for HowMany
# Usage: ./packaging/release.sh [patch|minor|major] [--dry-run]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERSION_TYPE="patch"
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        patch|minor|major)
            VERSION_TYPE="$1"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [patch|minor|major] [--dry-run]"
            echo ""
            echo "Options:"
            echo "  patch    Increment patch version (0.3.2 -> 0.3.3)"
            echo "  minor    Increment minor version (0.3.2 -> 0.4.0)"
            echo "  major    Increment major version (0.3.2 -> 1.0.0)"
            echo "  --dry-run Show what would be done without making changes"
            echo "  -h, --help Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown argument: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to increment version
increment_version() {
    local version=$1
    local type=$2
    
    # Remove 'v' prefix if present
    version=${version#v}
    
    IFS='.' read -ra PARTS <<< "$version"
    local major=${PARTS[0]}
    local minor=${PARTS[1]}
    local patch=${PARTS[2]}
    
    case $type in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Navigate to project directory
cd "$PROJECT_DIR"

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}Error: You have uncommitted changes. Please commit or stash them first.${NC}"
    git status --short
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep "^version = " Cargo.toml | sed 's/version = "\(.*\)"/\1/')
if [[ -z "$CURRENT_VERSION" ]]; then
    echo -e "${RED}Error: Could not find version in Cargo.toml${NC}"
    exit 1
fi

# Calculate new version
NEW_VERSION=$(increment_version "$CURRENT_VERSION" "$VERSION_TYPE")
NEW_TAG="v$NEW_VERSION"

echo -e "${BLUE}🚀 HowMany Release Process${NC}"
echo -e "${BLUE}=========================${NC}"
echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"
echo -e "New version:     ${GREEN}$NEW_VERSION${NC}"
echo -e "Version type:    ${YELLOW}$VERSION_TYPE${NC}"
echo -e "Git tag:         ${GREEN}$NEW_TAG${NC}"

if [[ "$DRY_RUN" == true ]]; then
    echo -e "\n${YELLOW}🔍 DRY RUN - No changes will be made${NC}"
    echo -e "\nWould perform the following actions:"
    echo -e "1. Update version in Cargo.toml to $NEW_VERSION"
    echo -e "2. Update version in Homebrew formula"
    echo -e "3. Run cargo check to validate"
    echo -e "4. Commit changes with message 'Bump version to $NEW_VERSION'"
    echo -e "5. Create git tag $NEW_TAG"
    echo -e "6. Push changes and tag to origin"
    echo -e "\nTo actually perform the release, run without --dry-run"
    exit 0
fi

# Confirmation prompt
echo -e "\n${YELLOW}Do you want to proceed with this release? (y/N)${NC}"
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo -e "${RED}Release cancelled${NC}"
    exit 1
fi

echo -e "\n${GREEN}✅ Starting release process...${NC}"

# Update version in Cargo.toml
echo -e "${BLUE}📝 Updating Cargo.toml...${NC}"
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update version in Homebrew formula if it exists
HOMEBREW_FORMULA="packaging/howmany.rb"
if [[ -f "$HOMEBREW_FORMULA" ]]; then
    echo -e "${BLUE}🍺 Updating Homebrew formula...${NC}"
    # Note: The SHA256 will be updated automatically by the GitHub Action
    sed -i.bak "s|archive/refs/tags/v[0-9]*\.[0-9]*\.[0-9]*\.tar\.gz|archive/refs/tags/$NEW_TAG.tar.gz|" "$HOMEBREW_FORMULA"
    rm -f "${HOMEBREW_FORMULA}.bak"
fi

# Run cargo check to ensure everything is valid
echo -e "${BLUE}🔍 Running cargo check...${NC}"
if ! cargo check; then
    echo -e "${RED}Error: cargo check failed. Please fix the issues and try again.${NC}"
    exit 1
fi

# Commit changes
echo -e "${BLUE}💾 Committing changes...${NC}"
git add Cargo.toml
if [[ -f "$HOMEBREW_FORMULA" ]]; then
    git add "$HOMEBREW_FORMULA"
fi
git commit -m "Bump version to $NEW_VERSION"

# Create and push tag
echo -e "${BLUE}🏷️  Creating git tag...${NC}"
git tag -a "$NEW_TAG" -m "Release $NEW_TAG"

# Push changes and tag
echo -e "${BLUE}📤 Pushing to origin...${NC}"
git push origin main
git push origin "$NEW_TAG"

echo -e "\n${GREEN}🎉 Release $NEW_VERSION completed successfully!${NC}"
echo -e "\n${BLUE}Next steps:${NC}"
echo -e "1. GitHub Actions will automatically:"
echo -e "   - Create a GitHub release"
echo -e "   - Build and upload binaries"
echo -e "   - Publish to crates.io"
echo -e "   - Update Homebrew formula"
echo -e "\n2. Monitor the GitHub Actions workflow at:"
echo -e "   https://github.com/GriffinCanCode/howmany/actions"
echo -e "\n3. Once complete, users can install with:"
echo -e "   ${GREEN}brew tap GriffinCanCode/howmany && brew install howmany${NC}"
echo -e "   ${GREEN}cargo install howmany${NC}" 