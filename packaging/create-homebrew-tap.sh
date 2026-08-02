#!/bin/bash

# Script to create a Homebrew tap for howmany
# This creates a separate repository for the Homebrew formula

set -e

REPO_NAME="homebrew-howmany"
GITHUB_USER="GriffinCanCode"
FORMULA_NAME="howmany"

echo "🍺 Creating Homebrew tap for $FORMULA_NAME"
echo "========================================="

# Check if we're in the right directory
if [ ! -f "packaging/howmany.rb" ]; then
    echo "❌ Error: Please run this script from the howmany-core directory"
    echo "   The packaging/howmany.rb file should exist"
    exit 1
fi

# Create temporary directory for the tap
TAP_DIR="/tmp/$REPO_NAME"
if [ -d "$TAP_DIR" ]; then
    echo "🧹 Cleaning up existing directory..."
    rm -rf "$TAP_DIR"
fi

echo "📁 Creating tap directory structure..."
mkdir -p "$TAP_DIR/Formula"

# Copy the formula
echo "📋 Copying formula..."
cp "packaging/howmany.rb" "$TAP_DIR/Formula/"

# Create README for the tap
echo "📝 Creating tap README..."
cat > "$TAP_DIR/README.md" << EOF
# Homebrew Tap for HowMany

This is a Homebrew tap for the [HowMany](https://github.com/$GITHUB_USER/howmany) code analysis tool.

## Installation

\`\`\`bash
brew tap $GITHUB_USER/howmany
brew install howmany
\`\`\`

## About HowMany

HowMany is a fast, intelligent code analysis tool with parallel processing, caching, and beautiful visualizations. It provides comprehensive statistics about files, lines of code, complexity, and development time estimates.

## Formula

The formula is located at \`Formula/howmany.rb\` and builds from the official GitHub releases.

## Updates

This tap is automatically updated when new versions of HowMany are released.
EOF

# Initialize git repository
echo "🔧 Setting up git repository..."
cd "$TAP_DIR"
git init
git add .
git commit -m "Initial commit: Add howmany formula"

echo ""
echo "✅ Homebrew tap created successfully!"
echo ""
echo "📍 Tap location: $TAP_DIR"
echo ""
echo "Next steps:"
echo "1. Create a new GitHub repository: https://github.com/$GITHUB_USER/$REPO_NAME"
echo "2. Add the remote and push:"
echo "   cd $TAP_DIR"
echo "   git remote add origin https://github.com/$GITHUB_USER/$REPO_NAME.git"
echo "   git branch -M main"
echo "   git push -u origin main"
echo ""
echo "3. Users can then install with:"
echo "   brew tap $GITHUB_USER/howmany"
echo "   brew install howmany"
echo ""
echo "4. To test locally:"
echo "   brew install --build-from-source $TAP_DIR/Formula/howmany.rb" 