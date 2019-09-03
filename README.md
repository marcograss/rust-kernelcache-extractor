# rust-kernelcache-extractor

Simple library and command line tool to extract iOS kernelcache and kpp, KISS.

Credits to Levin's joker tool.

`cargo build --release `

`./target/release/kcache_extract -i kernelcache.release.iphone -o kernelcache.release.iphone.decompressed -k kpp`

or

`cargo run --release  -- -i kernelcache.release.iphone -o kernelcache.release.iphone.decompressed -k kpp`

LICENSE should be GPL since the lzss decode function is taken from `https://github.com/pingw33n/vault13/blob/master/src/fs/dat/lzss.rs` , credits goes to them.

