# Make sure you have Docker/Podman first

echo "Installing Cross Compiler"
cargo install cross

echo "Adding Linux (x86) Target"
rustup target add i686-unknown-linux-gnu

echo "Building for Linux (x86)"
cross build --target i686-unknown-linux-gnu

echo "Testing Linux (x86)"
cross test --target i686-unknown-linux-gnu