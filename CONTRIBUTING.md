# Contributing

The `mutatis` project welcomes contributions for bug fixes, documentation
updates, new features, and more. Development is done through GitHub pull
requests. Feel free to [start a discussion][discussions] as well if you'd like
assistance in contributing or would just like to say hi.

[discussions]: https://github.com/fitzgen/mutatis/discussions

## Testing

To run all tests, make sure to enable all Cargo features, since none of them are
enabled by default:

```
$ cargo test --all-features
```

Depending on the exact part of the code base you are interested in exercising,
it may also make sense to test with only a subset of features enabled. For
example, to test the `no_std` support for the `alloc` crate, enable the `alloc`
feature but not the `std` feature:

```
$ cargo test --features alloc
```

To test under MIRI, first make sure `rustup` has installed a nightly toolchain
with MIRI for you:

```
$ rustup toolchain install nightly --component miri
```

Then, run the tests with MIRI enabled:

```
$ MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test --all-features
```

## Continuous Integration

All changes to `mutatis` are required to pass the CI suite powered by GitHub
Actions. Pull requests will automatically have checks performed and can only be
merged once all tests are passing.

CI checks currently include:

* Code is all formatted correctly. Run `cargo fmt` locally to fix this, if it is
  failing in CI.

* Tests pass on the current Rust stable, beta, and nightly channels, as well as
  on the minimum supported Rust version (MSRV).

* Tests pass when run under MIRI.

## License and Your Contributions

Licensed under dual MIT or Apache-2.0 at your choice.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
