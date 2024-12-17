# Make sure you have Docker/Podman first

echo "Installing Cross Compiler"
cargo install cross

echo "Adding Windows (x64) Target"
rustup target add i686-pc-windows-gnu

echo "Building for Windows (x86)"
cross build --target i686-pc-windows-gnu

echo "Testing Windows (x86)"
cross test --target i686-pc-windows-gnu