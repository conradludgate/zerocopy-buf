# zerocopy-buf

An integration to [`bytes`](https://docs.rs/bytes/1.0/bytes/index.html) with [`zerocopy@0.8`](https://docs.rs/zerocopy/0.8/zerocopy/index.html)

The minimum supported Rust version is 1.85. CI tests the maintained lockfile
directly on that compiler.

CI tests the minimum direct dependencies (`bytes` 1.8.0 and `zerocopy` 0.8.9)
and the latest compatible releases separately, including Miri and a `no_std`
bare-metal compilation check. The crate requires `alloc` but not `std`.
