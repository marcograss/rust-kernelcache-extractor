# rust-kernelcache-extractor

Simple library and command line tool to extract iOS kernelcache, KISS.

`cargo build --release `
`./target/debug/kcache_extract -i kernelcache.release.iphone11b -o kernelcache.release.iphone11b.decompressed`

or

`cargo run --release  -- -i kernelcache.release.iphone11b -o kernelcache.release.iphone11b.decompressed`
