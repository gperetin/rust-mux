## Mux Codec Tools for Rust

### [Documentation](http://bryce-anderson.github.io/rust-mux/index.html)

Some useful mux documentation can be found
[here](https://github.com/twitter/finagle/blob/master/finagle-mux/src/main/scala/com/twitter/finagle/mux/package.scala).

### Integ testing

To make it easier to test the client side of the library, there's a minimal
Mux server implementation in Scala in `tests/integration-server`. To run the
end-to-end tests in `tests/end_to_end.rs`, make sure the Scala server is
running:

    tests/integration-server/sbt 'runMain Server'

This will start listening on `localhost:8080`. Then run the tests with:

    cargo test --test end_to_end


### What is provided?
- Message types
- Message decoders
- Message encoders

___Note___: Everything is subject to change.

### What may come in the future?
- Session management (see [wip/session](https://github.com/bryce-anderson/rust-mux/tree/wip/session))
- Integration with mio

### License
Apache 2.0. See LICENSE file in the root directory.
