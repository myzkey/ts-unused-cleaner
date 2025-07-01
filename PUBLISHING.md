# Publishing TS Unused Cleaner to NPM

This document outlines the process for publishing the TS Unused Cleaner package to NPM.

## Prerequisites

1. **NPM Account**: You need an NPM account. Create one at [npmjs.com](https://www.npmjs.com/)
2. **NPM CLI**: Make sure you have npm installed and are logged in:
   ```bash
   npm login
   ```
3. **Rust Environment**: Ensure Rust is installed for building the binary
4. **Repository Setup**: Update the GitHub URLs in package.json to match your actual repository

## Pre-Publishing Checklist

### 1. Update Repository URLs

Edit `package.json` and update these fields with your actual GitHub repository:
```json
{
  "repository": {
    "type": "git",
    "url": "git+https://github.com/YOUR_USERNAME/ts-unused-cleaner.git"
  },
  "homepage": "https://github.com/YOUR_USERNAME/ts-unused-cleaner#readme",
  "bugs": {
    "url": "https://github.com/YOUR_USERNAME/ts-unused-cleaner/issues"
  },
  "author": {
    "name": "Your Name",
    "email": "your-email@example.com"
  }
}
```

### 2. Version Management

Update the version in `package.json`:
```bash
# For patch releases (bug fixes)
npm version patch

# For minor releases (new features)
npm version minor

# For major releases (breaking changes)
npm version major
```

### 3. Build and Test

```bash
# Build the project
npm run build

# Run tests
npm test

# Test the package locally
npm pack
npm install -g ts-unused-cleaner-1.0.0.tgz
```

## Publishing Process

### 1. Dry Run (Recommended)

Test what will be published without actually publishing:
```bash
npm publish --dry-run
```

This will show you:
- Which files will be included
- Package size
- Any warnings or errors

### 2. Publish to NPM

```bash
# Publish the package
npm publish

# For scoped packages (if using @username/package-name)
npm publish --access public
```

### 3. Verify Publication

After publishing, verify the package:
```bash
# Check package info
npm info ts-unused-cleaner

# Install globally to test
npm install -g ts-unused-cleaner

# Test the CLI
ts-unused-cleaner --help
```

## Post-Publishing

### 1. Create GitHub Release

1. Go to your GitHub repository
2. Click "Releases" → "Create a new release"
3. Tag version: `v1.0.0` (matching your npm version)
4. Release title: `v1.0.0 - Initial Release`
5. Add release notes describing features and changes

### 2. Update Documentation

- Update README.md with installation instructions
- Add badges for npm version and downloads
- Update any documentation links

## Package Structure

The published package includes:
```
ts-unused-cleaner/
├── bin/
│   └── ts-unused-cleaner          # Compiled binary
├── src/                          # Rust source code
├── Cargo.toml                    # Rust configuration
├── README.md                     # Documentation
├── LICENSE                       # MIT License
└── package.json                  # NPM configuration
```

## Troubleshooting

### Common Issues

1. **Binary not executable**: Ensure the binary has execute permissions
2. **Platform compatibility**: The binary is platform-specific; consider publishing separate packages for different platforms
3. **Large package size**: Use `.npmignore` to exclude unnecessary files

### Platform-Specific Publishing

For better cross-platform support, consider:
1. Building binaries for different platforms (Linux, macOS, Windows)
2. Using optional dependencies for platform-specific binaries
3. Creating separate packages like `ts-unused-cleaner-darwin`, `ts-unused-cleaner-linux`, etc.

## Automation

Consider setting up GitHub Actions for automated publishing:

1. On version tags
2. Automated testing before publishing  
3. Cross-platform binary building
4. Automated changelog generation

## Support

- NPM Package: https://www.npmjs.com/package/ts-unused-cleaner
- GitHub Issues: Update URL with your repository
- Documentation: Update with your repository URL