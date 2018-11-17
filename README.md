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


#### Client testing with a Scala server

To test the client implementation written in Rust against a Scala server,
clone [Finagle repo](https://github.com/twitter/finagle) and go to
`finagle/doc/src/sphinx/code/quickstart`.

Add `finagle-mux` dependency in `build.sbt` so it looks like this:

```
name := "quickstart"

version := "1.0"

scalaVersion := "2.12.1"

libraryDependencies += "com.twitter" %% "finagle-http" % "18.11.0"
libraryDependencies += "com.twitter" %% "finagle-mux" % "18.11.0"
```

Then change `Server.scala` to work with Mux:

```scala
import com.twitter.finagle.{Service, Mux}
import com.twitter.finagle.mux
import com.twitter.util.{Await, Future}

object Server extends App {
  val service = new Service[mux.Request, mux.Response] {
    def apply(req: mux.Request): Future[mux.Response] =
      Future.value(
        mux.Response.empty
      )
  }

  val server = Mux.serve(":8080", service)
  Await.ready(server)
}
```

This will start listening on `localhost:8080` and will always return an empty
Mux response.


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
