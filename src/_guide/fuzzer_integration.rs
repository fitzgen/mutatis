/*!

# Integrating Your Mutator with a Fuzzer

These are general steps for integrating your custom mutators into your fuzzing
or property-based testing framework of choice:

* Identify the framework's mechanism for customizing mutations, for example the
  [`libfuzzer_sys::fuzz_mutator!`][fuzz-mutator] macro.

* Implement that mechanism by:

  1. Converting the framework's raw bytes for the test case into your structured
     test case type, if necessary.

     Most fuzzers, including `libfuzzer`, don't know anything about your
     structured types, they just manipulate byte buffers. So you'll need to
     convert the raw bytes into your structured test case type. If you don't
     otherwise have a natural way of doing that, like if you're fuzzing a parser
     and could just run the parser on the raw data, then a quick-and-easy trick
     to to use `serde` and `bincode` to deserialize the raw bytes into your
     structured type.

  2. Run your `mutatis`-based custom mutator on the structured test case.

  3. Convert the structured test case back into raw bytes for the framework, if
     necessary.

     This is the inverse of step (i). If you used `serde` and `bincode` in step
     (i) you would also want to use them here in step (iii).

## Example: `libfuzzer`

While `mutatis` is agnostic of which fuzzing engine or property-testing
framework you use, here's an example of using `mutatis` to define a custom
mutator for [`libfuzzer-sys`][libfuzzer]. Integrating `mutatis` with other
fuzzing engines' APIs should look pretty similar, mutatis mutandis.

This example defines two different color representations, RGB and HSL, as well
as conversions between them. The fuzz target asserts that roundtripping an RGB
color to HSL and back to RGB correctly results in the original RGB color. The
fuzz mutator converts the fuzzer's bytes into an RGB color, mutates the RGB
color, and then updates the fuzzer's test case based on the mutated RGB color.

```rust,no_run
#[cfg(feature = "derive")]
# mod example {
use libfuzzer_sys::{fuzzer_mutate, fuzz_mutator, fuzz_target};
use mutatis::{mutators as m, Mutate, Session};

/// A red-green-blue color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Mutate)]
pub struct Rgb([u8; 3]);

impl Rgb {
    /// Create an RGB color from 3 bytes.
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        Rgb(bytes)
    }

    /// Get the underlying bytes for an RGB color.
    pub fn to_bytes(self) -> [u8; 3] {
        self.0
    }

    /// Convert this color from RGB to HSL.
    pub fn to_hsl(self) -> Hsl {
        todo!()
    }
}

/// A hue-saturation-lightness color.
pub struct Hsl {
    // ...
}

impl Hsl {
    /// Convert this color from HSL to RGB.
    pub fn to_rgb(self) -> Rgb {
        todo!()
    }
}

// The fuzz target: assert that RGB-to-HSL-to-RGB is the identity function.
fuzz_target!(|data| {
    let bytes = match data.first_chunk::<3>() {
        Some(b) => *b,
        None => return,
    };

    let rgb = Rgb::from_bytes(bytes);
    let hsl = rgb.to_hsl();
    let rgb2 = hsl.to_rgb();

    assert_eq!(rgb, rgb2);
});

// The custom mutator: create an RGB color from the fuzzer's raw data, mutate
// it, update the fuzzer's raw data based on that mutated RGB color.
fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
    let bytes = match data.first_chunk::<3>() {
        Some(b) => *b,
        // If we don't have enough bytes to mutate ourselves, use the fuzzer's
        // default mutation strategies.
        None => return fuzzer_mutate(data, size, max_size),
    };

    let mut rgb = Rgb::from_bytes(bytes);

    // Configure the mutation with the seed that libfuzzer gave us.
    let mut session = Session::new().seed(seed.into());

    // Mutate `rgb` using its default, derived mutator!
    match session.mutate(&mut rgb) {
        Ok(()) => {
            // Update the fuzzer's raw data based on the mutated RGB color.
            let new_bytes = rgb.to_bytes();
            let new_size = std::cmp::min(max_size, new_bytes.len());
            data[..new_size].copy_from_slice(&bytes[..new_size]);
            new_size
        }
        Err(_) => {
            // If we failed to mutate the test case, fall back to the fuzzer's
            // default mutation strategies.
            return fuzzer_mutate(data, size, max_size);
        }
    }
});
# }
```

[libfuzzer]: https://crates.io/crates/libfuzzer-sys
[fuzz-mutator]: https://docs.rs/libfuzzer-sys/latest/libfuzzer_sys/macro.fuzz_mutator.html

 */
