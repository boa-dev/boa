# boa_icu_provider

`boa_icu_provider` generates and defines the [ICU4X](https://github.com/unicode-org/icu4x) data provider
used in the Boa engine to enable internationalization functionality.

## Datagen

To regenerate the data:

```bash
$ cargo run --bin boa-datagen --features bin
```