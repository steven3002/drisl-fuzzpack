#!/bin/bash
set -e


cd "$(dirname "$0")/.."

echo "==================================================="
echo "    Provisioning DRISL FuzzPack Environment"
echo "==================================================="

# Setup Node.js (atcute)
echo -e "\n[*] Installing JS/TS Dependencies (@atcute)..."
cd adapters/atcute
npm install
cd ../..


# Setup Python (python-libipld)
echo -e "\n[*] Setting up Python Virtual Environment (libipld)..."
cd adapters/python-libipld
python3 -m venv venv
./venv/bin/pip install libipld
cd ../..

# Setup Go (go-dasl)
echo -e "\n[*] Building Go Adapter (go-dasl)..."
cd adapters/go-dasl
go build -o go-adapter main.go
cd ../..

# Setup Rust (Coordinator & serde_ipld)
echo -e "\n[*] Building Rust Coordinator & Adapter..."
cd coordinator
cargo build
cd ..

echo -e "\n✅ Environment successfully provisioned. You can now run FuzzPack!"
