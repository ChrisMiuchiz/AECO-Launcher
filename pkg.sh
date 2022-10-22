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


# Make target Windows dirs
mkdir -p ./packages/i686-pc-windows-gnu/bin
mkdir -p ./packages/x86_64-pc-windows-gnu/bin
mkdir -p ./packages/i686-pc-windows-gnu/standalone
mkdir -p ./packages/x86_64-pc-windows-gnu/standalone

# Use resource hacker so we can add Windows icons from unixlikes...
if [ ! -d "resource_hacker" ]; then
    curl http://angusj.com/resourcehacker/resource_hacker.zip --output resource_hacker.zip
    unzip resource_hacker.zip -d resource_hacker
    rm resource_hacker.zip
fi

# Clear target dir
cargo clean

# Build 32 bit
cross build --release --target i686-pc-windows-gnu

# Copy 32 bit bin
cp ./target/i686-pc-windows-gnu/release/aeco-launcher.exe ./packages/i686-pc-windows-gnu/bin

# Add icon to 32 bit binary
wine resource_hacker/ResourceHacker.exe -open ./packages/i686-pc-windows-gnu/bin/aeco-launcher.exe -save ./packages/i686-pc-windows-gnu/bin/aeco-launcher.exe -action addskip -res assets/atomix-eco-saga10.ico -mask ICONGROUP,MAINICON,

# Create 32 bit zip
mkdir -p ./packages/i686-pc-windows-gnu/standalone/Atomix-ECO
cp ./packages/i686-pc-windows-gnu/bin/aeco-launcher.exe ./packages/i686-pc-windows-gnu/standalone/Atomix-ECO
cd ./packages/i686-pc-windows-gnu/standalone/
zip -r Atomix-ECO.zip ./Atomix-ECO
rm -r ./Atomix-ECO
cd -

# Clear target dir
cargo clean

# Build 64 bit
cross build --release --target x86_64-pc-windows-gnu

# Copy 64 bit bin
cp ./target/x86_64-pc-windows-gnu/release/aeco-launcher.exe ./packages/x86_64-pc-windows-gnu/bin

# Add icon to 64 bit binary
wine resource_hacker/ResourceHacker.exe -open ./packages/x86_64-pc-windows-gnu/bin/aeco-launcher.exe -save ./packages/x86_64-pc-windows-gnu/bin/aeco-launcher.exe -action addskip -res assets/atomix-eco-saga10.ico -mask ICONGROUP,MAINICON,

# Create 64 bit zip
mkdir -p ./packages/x86_64-pc-windows-gnu/standalone/Atomix-ECO
cp ./packages/x86_64-pc-windows-gnu/bin/aeco-launcher.exe ./packages/x86_64-pc-windows-gnu/standalone/Atomix-ECO
cd ./packages/x86_64-pc-windows-gnu/standalone/
zip -r Atomix-ECO.zip ./Atomix-ECO
rm -r ./Atomix-ECO
cd -