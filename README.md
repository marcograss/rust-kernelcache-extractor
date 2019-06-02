# rust-kernelcache-extractor

Simple library and command line tool to extract iOS kernelcache, KISS.

`cargo build --release `
`./target/release/kcache_extract -i kernelcache.release.iphone11b -o kernelcache.release.iphone11b.decompressed`

or

`cargo run --release  -- -i kernelcache.release.iphone11b -o kernelcache.release.iphone11b.decompressed`

LICENSE should be GPL since the lzss decode function is taken from `https://github.com/pingw33n/vault13/blob/master/src/fs/dat/lzss.rs` , credits goes to them.

