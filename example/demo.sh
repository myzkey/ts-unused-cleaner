#!/bin/bash

echo "🚀 TS Unused Cleaner Demo"
echo "=========================="
echo ""

# Build the tool if needed
echo "📦 Building TS Unused Cleaner..."
cd ..
cargo build --release
cd example

echo ""
echo "📂 Example project structure:"
find react/src -name "*.ts" -o -name "*.tsx" | head -10

echo ""
echo "🔍 Running TS Unused Cleaner with different options..."
echo ""

echo "1️⃣ Basic scan (components only):"
(cd react && ../../target/release/ts-unused-cleaner)
echo ""

echo "2️⃣ Scan all element types:"
(cd react && ../../target/release/ts-unused-cleaner --all)
echo ""

echo "3️⃣ Verbose output:"
(cd react && ../../target/release/ts-unused-cleaner --all --verbose)
echo ""

echo "4️⃣ Using custom config file:"
(cd react && ../../target/release/ts-unused-cleaner --config ../tuc.config.json)
echo ""

echo "5️⃣ Strict mode (would exit with error if unused elements found):"
(cd react && ../../target/release/ts-unused-cleaner --all --strict) || echo "⚠️  Strict mode detected unused elements"
echo ""

echo "✅ Demo completed! Check the results above to see unused elements detected."