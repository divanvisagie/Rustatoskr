# Rustatoskr
Life's too short to use Go interfaces

Rustatoskr is a rewrite of Ratatoskr in Rust. Part of this was simply for research purposes to see the difference between the memory usage between a go and rust implementation of the same application. The other part was to see how much of a difference the type system makes in terms of code quality and readability.

Writing in rust also lets us make use of the advances in local AI that have been made that make use of native integration such as local tokenization which seem to be limited to python and rust at the moment.

![Rustatoskr](docs/logo.jpg)

## Continous dev:

### Windows
```powershell
$env:RUST_LOG="trace"; cargo watch -c -x run
```

### Linux
```sh
RUST_LOG=trace cargo watch -c -x run
```
