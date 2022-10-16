#!/bin/bash

linux_wrapper_template=$(cat ./linux-wrapper-template.sh)

# Clear target dir
cargo clean

# Make target Linux dirs
mkdir -p ./packages/x86_64-unknown-linux-gnu/bin
mkdir -p ./packages/x86_64-unknown-linux-gnu/wrapper
mkdir -p ./packages/x86_64-unknown-linux-gnu/standalone

# Build
cross build --release --target x86_64-unknown-linux-gnu

# Copy bin
cp ./target/x86_64-unknown-linux-gnu/release/aeco-launcher ./packages/x86_64-unknown-linux-gnu/bin

# Create wrapper script
encoded_bin=$(cat ./packages/x86_64-unknown-linux-gnu/bin/aeco-launcher | gzip | base64 -w0)
echo "${linux_wrapper_template/STAMP_LAUNCHER_HERE/$encoded_bin}" > ./packages/x86_64-unknown-linux-gnu/wrapper/atomix-eco-saga10

# Create tar.gz
mkdir -p ./packages/x86_64-unknown-linux-gnu/standalone/Atomix-ECO
cp ./packages/x86_64-unknown-linux-gnu/bin/aeco-launcher ./packages/x86_64-unknown-linux-gnu/standalone/Atomix-ECO
cd ./packages/x86_64-unknown-linux-gnu/standalone
tar --numeric-owner -czf Atomix-ECO.tar.gz Atomix-ECO/
cd -
rm -r ./packages/x86_64-unknown-linux-gnu/standalone/Atomix-ECO

# Create deb
cp -r ./deb ./packages/x86_64-unknown-linux-gnu/
mkdir -p ./packages/x86_64-unknown-linux-gnu/deb/atomix-eco-saga10_amd64/usr/local/bin/
cp ./packages/x86_64-unknown-linux-gnu/wrapper/atomix-eco-saga10 ./packages/x86_64-unknown-linux-gnu/deb/atomix-eco-saga10_amd64/usr/local/bin/
chmod 775 ./packages/x86_64-unknown-linux-gnu/deb/atomix-eco-saga10_amd64/usr/local/bin/atomix-eco-saga10
mkdir -p ./packages/x86_64-unknown-linux-gnu/deb/atomix-eco-saga10_amd64/usr/share/icons/hicolor/128x128/apps/
cp ./assets/atomix-eco-saga10.png ./packages/x86_64-unknown-linux-gnu/deb/atomix-eco-saga10_amd64/usr/share/icons/hicolor/128x128/apps/
cd ./packages/x86_64-unknown-linux-gnu/deb/
dpkg-deb --build --root-owner-group ./atomix-eco-saga10_amd64
rm -r ./atomix-eco-saga10_amd64
cd -