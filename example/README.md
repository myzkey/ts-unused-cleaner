# TS Unused Cleaner Example Project

This example demonstrates how to use the TS Unused Cleaner tool to detect unused TypeScript/JavaScript code including React components, types, interfaces, functions, variables, and enums.

## React Application

This example includes a functional React app that you can run and interact with while demonstrating the unused code detection capabilities.

### Running the React App

```bash
# Navigate to React app
cd react

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Type checking
npm run type-check
```

The app will be available at http://localhost:3000

## Project Structure

```
example/
├── react/                      # React application
│   ├── src/
│   │   ├── components/
│   │   │   ├── button.tsx      # ✅ Used in Home.tsx
│   │   │   ├── card.tsx        # ✅ Used in Home.tsx
│   │   │   ├── spinner.tsx     # ❌ Unused component
│   │   │   └── unused-modal.tsx # ❌ Unused component
│   │   ├── pages/
│   │   │   └── Home.tsx        # ✅ Used in App.tsx
│   │   ├── types/
│   │   │   └── api.ts          # Mixed: some types used, some unused
│   │   ├── utils/
│   │   │   └── api.ts          # Mixed: some functions used, some unused
│   │   ├── App.tsx             # Main app component
│   │   ├── main.tsx            # React entry point
│   │   └── index.css           # Styles
│   ├── package.json            # React dependencies
│   ├── vite.config.ts          # Vite configuration
│   ├── tsconfig.json           # TypeScript configuration
│   └── index.html              # HTML entry point
├── tuc.config.json             # TS Unused Cleaner configuration
├── demo.sh                     # Demo script
└── README.md                   # This file
```

## Expected Unused Elements

The example includes intentionally unused elements to demonstrate TS Unused Cleaner's detection capabilities:

### Unused Components
- `UnusedModal` - Modal component not imported anywhere
- `Spinner` - Loading spinner component not used

### Unused Types/Interfaces
- `UnusedDataType` - Type definition not referenced
- `Post` - Interface not used in any component

### Unused Functions/Variables
- `unusedHelper` - Utility function not called
- `UNUSED_CONSTANT` - Constant not referenced
- `calculateTotal` - Function not imported/used

### Unused Enums
- `UnusedStatus` - Enum not referenced anywhere

## Running the Demo

1. **Build React Unused Cleaner** (from project root):
   ```bash
   cargo build --release
   ```

2. **Run the demo script**:
   ```bash
   cd example
   ./demo.sh
   ```

3. **Or run React Unused Cleaner manually**:
   ```bash
   # Basic scan (components only)
   ../target/release/ts-unused-cleaner
   
   # Scan all element types
   ../target/release/ts-unused-cleaner --all
   
   # Verbose output with performance info
   ../target/release/ts-unused-cleaner --all --verbose
   
   # Using custom config file
   ../target/release/ts-unused-cleaner --config tuc.config.json
   
   # Strict mode (exits with error if unused found)
   ../target/release/ts-unused-cleaner --all --strict
   ```

## Configuration

The example includes a `tuc.config.json` file that demonstrates:
- Custom search directories
- File exclusion patterns  
- Detection type toggles
- CI/CD integration settings

## Learning Points

This example showcases:
- How TS Unused Cleaner detects different types of unused elements
- Configuration options and their effects
- Real-world project structure with mixed used/unused code
- Performance benefits of the Rust implementation
- A functional React application that can be run and tested