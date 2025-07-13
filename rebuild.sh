#!/bin/bash

# Rebuild script for howmany - rebuilds cargo and reestablishes symlink

set -e  # Exit on any error

echo "🔨 Rebuilding howmany..."
echo "========================"

# Clean previous build
echo "🧹 Cleaning previous build..."
cargo clean

# Build the project
echo "🏗️  Building project..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Create symlink (remove existing one first if it exists)
SYMLINK_PATH="/usr/local/bin/howmany"
BINARY_PATH="$(pwd)/target/release/howmany"

echo "🔗 Setting up symlink..."

# Remove existing symlink if it exists
if [ -L "$SYMLINK_PATH" ]; then
    echo "   Removing existing symlink..."
    sudo rm "$SYMLINK_PATH"
fi

# Create new symlink
echo "   Creating symlink: $SYMLINK_PATH -> $BINARY_PATH"
sudo ln -s "$BINARY_PATH" "$SYMLINK_PATH"

# Verify the symlink works
if [ -L "$SYMLINK_PATH" ] && [ -e "$SYMLINK_PATH" ]; then
    echo "✅ Symlink created successfully!"
    echo "🎉 You can now use 'howmany' from anywhere!"
    echo ""
    echo "Testing the installation:"
    howmany --version
else
    echo "❌ Failed to create symlink!"
    exit 1
fi

echo ""
echo "🚀 Rebuild complete! The enhanced howmany is ready to use."
echo "   Try: howmany interactive"
echo "   Or:  howmany count --verbose" 