# `emwin-tg`

A Rust client for the [NWS Emergency Managers Weather Information Network](https://www.weather.gov/emwin/)
telecommunications gateway.

EMWIN is [one of several](https://www.weather.gov/nwws/dissemination) platforms through which the National Weather
Service distributes text products. EMWIN is transmitted via radio to [GOES
satellites](https://noaasis.noaa.gov/GOES/HRIT/broadcast.html) for re-broadcast, and published as files on the
[NWS telecommunications gateway](https://www.weather.gov/tg/anonymous) service.

The NWS telecommunications gateway permits anonymous access from the general public over the Internet. Usage of this
crate therefore requires no setup.

## Features

* `#![forbid(unsafe_code)]`
* Pure Rust
* Async (using [Tokio](https://tokio.rs))
* WebAssembly support (using [wasm-pack](https://rustwasm.github.io/wasm-pack/))

## Rust Example

```rust
let mut stream = emwin_tg::TextStream::new();

while let Some(event) = stream.next().await {
    match event {
        Ok(product) => {
            // Handle the product
            println!("{}: {}", product.filename, product.into_string_lossy());
        },
        Err(error) => {
            // Stream continues, automatically retrying as needed
            eprintln!("uh oh: {}", error);
        },
    }
}
```

## JavaScript example

```javascript
import EmwinTg;
```