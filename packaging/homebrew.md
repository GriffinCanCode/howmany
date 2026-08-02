# Homebrew Publishing Guide for HowMany

This guide explains how to publish the `howmany` tool to Homebrew, making it easily installable via `brew install howmany`.

## Files

- `howmany.rb` - The Homebrew formula for the `howmany` package

## Publishing Options

### Option 1: Submit to homebrew-core (Recommended for Popular Tools)

To get `howmany` into the main Homebrew repository:

1. **Prerequisites**:
   - The tool should be well-known and widely used
   - Must have a stable release with proper versioning
   - Must follow Homebrew's guidelines

2. **Steps**:
   ```bash
   # Fork homebrew-core
   git clone https://github.com/Homebrew/homebrew-core.git
   cd homebrew-core
   
   # Create the formula
   cp /path/to/howmany-core/packaging/howmany.rb Formula/howmany.rb
   
   # Test the formula
   brew install --build-from-source ./Formula/howmany.rb
   brew test howmany
   brew audit --strict howmany
   
   # Create PR
   git checkout -b howmany
   git add Formula/howmany.rb
   git commit -m "howmany: add new formula"
   git push origin howmany
   ```

3. **Create a Pull Request** to [homebrew-core](https://github.com/Homebrew/homebrew-core)

### Option 2: Create a Homebrew Tap (Easier Alternative)

Create your own tap for immediate availability:

1. **Create a new repository** named `homebrew-howmany`:
   ```bash
   # Create repository at https://github.com/GriffinCanCode/homebrew-howmany
   git clone https://github.com/GriffinCanCode/homebrew-howmany.git
   cd homebrew-howmany
   
   # Copy the formula
   mkdir -p Formula
   cp /path/to/howmany-core/packaging/howmany.rb Formula/howmany.rb
   
   # Commit and push
   git add Formula/howmany.rb
   git commit -m "Add howmany formula"
   git push origin main
   ```

2. **Users can then install with**:
   ```bash
   brew tap GriffinCanCode/howmany
   brew install howmany
   ```

## Formula Details

The current formula (`howmany.rb`) includes:

- **Source**: GitHub release tarball (v0.3.2)
- **Dependencies**: Rust (build-time only)
- **Build Process**: Cargo build in the `howmany-core` subdirectory
- **Tests**: Version check, help check, and basic functionality test

## Testing the Formula

Before publishing, test the formula locally:

```bash
# Install from local formula
brew install --build-from-source packaging/howmany.rb

# Run tests
brew test howmany

# Audit for issues
brew audit --strict packaging/howmany.rb

# Uninstall after testing
brew uninstall howmany
```

## Updating the Formula

When releasing a new version:

1. **Create a new GitHub release** with a git tag (e.g., `v0.3.3`)

2. **Update the formula**:
   ```bash
   # Get the new SHA256
   curl -sL https://github.com/GriffinCanCode/howmany/archive/refs/tags/v0.3.3.tar.gz | shasum -a 256
   
   # Update howmany.rb with:
   # - New version in the URL
   # - New SHA256 hash
   # - Update version in tests if needed
   ```

3. **Test and submit** the updated formula

## License Considerations

The current license is "Griffin-1.0" (custom license with attribution requirements). This may need to be:

- Registered as a custom license identifier with Homebrew
- Or changed to a standard license (MIT, Apache-2.0, etc.) for easier acceptance

## Maintenance

Once published, the formula will need updates for:

- New releases
- Dependency changes
- Build system changes
- macOS compatibility updates

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating Homebrew Taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap) 