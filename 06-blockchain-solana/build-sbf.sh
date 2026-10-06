#!/usr/bin/env bash
# Build Phase 6's programs for the real Solana VM (optional track).
#
#   ./build-sbf.sh              the reference solutions -> target/deploy/solution/<program>.so
#   ./build-sbf.sh mine         your exercise crates    -> target/deploy/mine/<program>.so
#   ./build-sbf.sh mine 26      just one module
#
# Needs the Solana toolchain (`cargo build-sbf`, see GETTING_STARTED.md).
# Then run the on-chain tests, which load these files into LiteSVM:
#
#   cargo test -p m26-solana-basics-tests --features sbf
#   cargo test -p m26-solana-basics-tests --features sbf,mine
set -euo pipefail
cd "$(dirname "$0")"

which="${1:-solution}"
only="${2:-}"
case "$which" in
solution) crate_dir=solution ;;
mine) crate_dir=exercise ;;
*)
    echo "usage: $0 [solution|mine] [module number]" >&2
    exit 2
    ;;
esac

# module directory, then each program as <feature>[=<output name>]. A crate
# with a single program (Anchor) uses "-" for "no feature".
programs=(
    "26-solana-basics hello notes vault"
    "27-anchor-framework -=anchor_lab"
    "28-smart-contracts multisig escrow staking bank bank-secure"
    "29-nfts-tokens metadata vending nft-staking"
    "30-defi-protocols amm oracle lending"
)

out="target/deploy/$which"
mkdir -p "$out"
for line in "${programs[@]}"; do
    read -r module features <<<"$line"
    [[ -n "$only" && "$module" != "$only"* ]] && continue
    manifest="$module/$crate_dir/Cargo.toml"
    for feature in $features; do
        name="${feature#*=}"
        feature="${feature%%=*}"
        tmp="target/deploy/.build/$which-$name"
        args=(--manifest-path "$manifest" --sbf-out-dir "$tmp")
        [[ "$feature" != "-" ]] && args+=(--features "$feature")
        echo "==> $module: $name"
        cargo build-sbf "${args[@]}" -- --lib
        # build-sbf names the file after the crate; name it after the program.
        cp "$tmp"/*.so "$out/$name.so"
    done
done
echo "built: $(ls "$out" | tr '\n' ' ')"
