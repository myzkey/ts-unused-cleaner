# TS Unused Finder

[![Crates.io](https://img.shields.io/crates/v/ts-unused-finder)](https://crates.io/crates/ts-unused-finder)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A blazingly fast tool to detect unused TypeScript/JavaScript code including React components, types, interfaces, functions, variables, and enums in your codebase.

## Features

- 🚀 **Lightning Fast** - Written in Rust with parallel processing
- 🎯 **Comprehensive Detection** - Components, types, interfaces, functions, variables, enums
- ⚙️ **Configurable** - Flexible configuration with JSON files
- 📊 **Detailed Reports** - Clear output with usage statistics
- 🔧 **CI/CD Ready** - Exit codes and thresholds for automation
- 📁 **Monorepo Support** - Handles complex project structures
- ⚛️ **React Optimized** - Special patterns for React components and hooks

## Installation

### From NPM (Recommended)

```bash
npm install -g ts-unused-finder
# or
npx ts-unused-finder
```

### From Cargo

```bash
cargo install ts-unused-finder
```

### From Source

```bash
git clone https://github.com/your-username/ts-unused-finder
cd ts-unused-finder
cargo build --release
# Binary will be available at ./target/release/ts-unused-finder
```

## Quick Start

```bash
# Detect unused React components (default)
ts-unused-finder

# Detect all element types
ts-unused-finder --all

# Use verbose output
ts-unused-finder --all --verbose

# Strict mode (exit with error if unused found)
ts-unused-finder --all --strict
```

## Usage

### Basic Commands

```bash
# Basic scan (components only)
ts-unused-finder

# Scan specific element types
ts-unused-finder --types --interfaces --functions

# Scan everything
ts-unused-finder --all

# Verbose output with performance info
ts-unused-finder --verbose

# Quiet mode (errors only)
ts-unused-finder --quiet

# Custom number of parallel jobs
ts-unused-finder --jobs 8

# Use custom config file
ts-unused-finder --config path/to/tuf.config.json

# Strict mode for CI/CD
ts-unused-finder --strict
```

### Detection Types

| Flag | Description | Example |
|------|-------------|---------|
| (default) | React components | `function MyComponent()`, `const Button = () =>` |
| `--types` | TypeScript type definitions | `type User = {...}` |
| `--interfaces` | TypeScript interfaces | `interface ApiResponse {...}` |
| `--functions` | Function declarations | `function helper()`, `const utils = () =>` |
| `--variables` | Variable/constant declarations | `const API_URL = "..."`, `let config = {...}` |
| `--enums` | TypeScript enums | `enum Status {...}` |
| `--all` | All of the above | |

## Configuration

Create a `tuf.config.json` file in your project root:

```json
{
  "search_dirs": ["src", "components", "lib"],
  "exclude_patterns": [
    "node_modules",
    "*.test.ts",
    "*.test.tsx",
    "*.spec.ts",
    "*.spec.tsx",
    "*.stories.ts",
    "*.stories.tsx",
    "*.d.ts",
    "dist",
    "build",
    ".next",
    "coverage",
    "__tests__",
    "tests"
  ],
  "detection_types": {
    "components": true,
    "types": true,
    "interfaces": true,
    "functions": true,
    "variables": true,
    "enums": true
  },
  "ci": {
    "max_unused_elements": 10,
    "fail_on_exceed": true,
    "log_level": "warn"
  }
}
```

### Configuration Files

TS Unused Finder looks for configuration files in this order:

1. Custom config file specified via `--config` flag
2. `tuf.config.json` in the current directory

If no configuration file is found, the tool will use default settings.

## Use Cases

### React/Next.js Projects
- Unused React components and hooks
- Dead TypeScript types and interfaces
- Orphaned utility functions
- Unused constants and enums

### TypeScript Libraries
- Unused exported types
- Dead utility functions
- Orphaned interfaces
- Unused enum values

### Node.js Applications
- Unused helper functions
- Dead configuration objects
- Orphaned type definitions
- Unused middleware

### Monorepos
- Cross-package unused exports
- Dead shared utilities
- Unused design system components
- Orphaned type definitions

## CI/CD Integration

### GitHub Actions

```yaml
name: Check Unused Code
on: [push, pull_request]

jobs:
  unused-check:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install TS Unused Finder
      run: npm install -g ts-unused-finder
    - name: Check for unused code
      run: ts-unused-finder --all --strict
```

### Exit Codes

- `0` - Success (no unused elements or within threshold)
- `1` - Error (unused elements found in strict mode or above threshold)

## Example Output

```
🔍 TS Unused Finder - Scanning for unused elements...

📊 Detection Results:
┌─────────────┬───────┬──────┬────────┐
│ Type        │ Total │ Used │ Unused │
├─────────────┼───────┼──────┼────────┤
│ Components  │    15 │   12 │      3 │
│ Types       │     8 │    6 │      2 │
│ Interfaces  │     5 │    4 │      1 │
│ Functions   │    20 │   18 │      2 │
│ Variables   │    10 │    8 │      2 │
│ Enums       │     3 │    2 │      1 │
└─────────────┴───────┴──────┴────────┘

❌ Unused Elements Found:

React Components:
  • UnusedModal (src/components/UnusedModal.tsx)
  • OldButton (src/components/OldButton.tsx)
  • DeprecatedCard (src/components/DeprecatedCard.tsx)

Types:
  • UnusedDataType (src/types/api.ts:15)
  • LegacyUser (src/types/user.ts:8)

Functions:
  • unusedHelper (src/utils/helpers.ts:42)
  • deprecatedFormatter (src/utils/format.ts:18)

⏱️  Execution time: 0.12s
🚀 Accelerated by Rust implementation
```

## Performance

TS Unused Finder is designed for speed:

- **Parallel Processing** - Utilizes all CPU cores via Rayon
- **Optimized Regex** - Compiled patterns with efficient matching
- **Memory Efficient** - Streaming file processing
- **Rust Performance** - Native speed with zero-cost abstractions
- **Thread Pool Configuration** - Configurable via `--jobs` flag

Typical performance on large codebases:
- **1000+ files**: ~0.5-2 seconds
- **10,000+ files**: ~5-15 seconds
- **Monorepos with 50k+ files**: ~30-60 seconds

Performance can be tuned using:
```bash
# Use 8 parallel jobs
ts-unused-finder --jobs 8

# Maximum parallelism (uses all CPU cores)
ts-unused-finder --jobs $(nproc)
```

## Supported Patterns

### React Components
- `export default function ComponentName`
- `export const ComponentName = () =>`
- `export const ComponentName = React.memo()`
- `export const ComponentName = forwardRef()`
- `const ComponentName = React.forwardRef()`

### TypeScript Types
- `export type TypeName = ...`
- `type TypeName = ...`

### Interfaces
- `export interface InterfaceName`
- `interface InterfaceName`

### Functions
- `export function functionName`
- `export const functionName = () =>`
- `function functionName`
- `const functionName = async () =>`

### Variables
- `export const CONSTANT_NAME`
- `export let variableName`
- `const CONSTANT_NAME`

### Enums
- `export enum EnumName`
- `enum EnumName`

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development

```bash
# Clone the repository
git clone https://github.com/your-username/ts-unused-finder
cd ts-unused-finder

# Build the project
cargo build
# or via npm
npm run build

# Run tests
cargo test
# or via npm
npm test

# Format code
cargo fmt
# or via npm
npm run fmt

# Run example
cd example
./demo.sh
```

### Development Dependencies

The project uses several Rust crates for development:
- `clap` - Command line argument parsing
- `serde` - Serialization/deserialization
- `regex` - Pattern matching
- `rayon` - Parallel processing
- `walkdir` - File system traversal
- `colored` - Terminal output coloring
- `indicatif` - Progress bars

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for maximum performance
- Uses [Rayon](https://github.com/rayon-rs/rayon) for parallel processing
- Optimized for TypeScript/JavaScript ecosystems including React, Next.js, Node.js, and more