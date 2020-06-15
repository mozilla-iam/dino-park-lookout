export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH
export RUST_VERSION=1.44.0

set -eux
dpkgArch="$(dpkg --print-architecture)"
case "${dpkgArch##*-}" in
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='ad1f8b5199b3b9e231472ed7aa08d2e5d1d539198a15c5b1e53c746aad81d27b' ;;
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='6c6c3789dabf12171c7f500e06d21d8004b5318a5083df8b0b02c0e5ef1d017b' ;;
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='26942c80234bac34b3c1352abbd9187d3e23b43dae3cf56a9f9c1ea8ee53076d' ;;
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='27ae12bc294a34e566579deba3e066245d09b8871dc021ef45fc715dced05297' ;;
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;;
esac
url="https://static.rust-lang.org/rustup/archive/1.21.1/${rustArch}/rustup-init"
wget "$url"
echo "${rustupSha256} *rustup-init" | sha256sum -c -
chmod +x rustup-init
./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION
rm rustup-init
chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup --version
cargo --version
rustc --version

rustup target add x86_64-unknown-linux-musl
apt-get update
apt-get install -y musl-tools
HELM_INSTALL_DIR=/bin
curl https://raw.githubusercontent.com/helm/helm/master/scripts/get | bash
curl -L -o myke https://github.com/fiji-flo/myke/releases/download/0.9.11/myke-0.9.11-x86_64-unknown-linux-musl
chmod +x ./myke
mv ./myke /bin/myke