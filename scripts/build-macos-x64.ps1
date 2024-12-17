# Make sure you have Docker/Podman first

echo "Installing Cross Compiler"
cargo install cross

echo "Adding macOS (x64) Target"
rustup target add x86_64-apple-darwin

echo "Building macOS (x64) Target"
cross build --target x86_64-apple-darwin