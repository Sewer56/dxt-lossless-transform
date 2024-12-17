# Make sure you have Docker/Podman first

echo "Installing Cross Compiler"
cargo install cross

echo "Adding Windows (x64) Target"
rustup target add x86_64-pc-windows-gnu

echo "Building for Windows (x64)"
cross build --target x86_64-pc-windows-gnu

echo "Testing Windows (x64)"
cross test --target x86_64-pc-windows-gnu