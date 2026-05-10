# Maintainer: sreevarshan <sreevarshan1511@gmail.com>
pkgname=metapak
pkgver=0.1.0
pkgrel=1
pkgdesc="A unified TUI for Arch Linux package management (Pacman + AUR)"
arch=('x86_64')
url="https://github.com/sreevarshan-xenoz/metapak"
license=('MIT')
depends=('gcc-libs' 'openssl' 'pacman')
makedepends=('cargo' 'git')
optdepends=('paru: AUR support' 'yay: AUR support')
source=("git+$url.git")
sha256sums=('SKIP')

prepare() {
    cd "$pkgname"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$pkgname"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

package() {
    cd "$pkgname"
    install -Dm755 target/release/metapak "$pkgdir/usr/bin/metapak"
    install -Dm644 metapak.desktop "$pkgdir/usr/share/applications/metapak.desktop"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
