## Mux Codec Tools for Rust

### [Documentation](http://bryce-anderson.github.io/rust-mux/index.html)

Some useful mux documentation can be found
[here](https://github.com/twitter/finagle/blob/master/finagle-mux/src/main/scala/com/twitter/finagle/mux/package.scala).

### Testing

To run the example in `examples`, use nc to listen on a port and return a
hardcoded Rdispatch message:

    python -c 'import binascii, sys; sys.stdout.write(binascii.unhexlify("0000000cfe0000020000006c65676974"))' | nc -l 9000 | xxd

and run `cargo run`.

For a ping message use this `nc` line:

    python -c 'import binascii, sys; sys.stdout.write(binascii.unhexlify("00000004BF000002"))' | nc -l 9000 | xxd


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
