# Make sure you have Docker/Podman first

echo "Installing Cross Compiler"
cargo install cross

echo "Adding Linux (x64) Target"
rustup target add x86_64-unknown-linux-gnu

echo "Building for Linux (x64)"
cross build --target x86_64-unknown-linux-gnu

echo "Testing Linux (x64)"
cross test --target x86_64-unknown-linux-gnu