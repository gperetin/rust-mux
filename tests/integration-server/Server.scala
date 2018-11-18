import com.twitter.finagle.{Service, Mux}
import com.twitter.finagle.mux
import com.twitter.io.Buf
import com.twitter.util.{Await, Future}

object Server extends App {
  val service = new Service[mux.Request, mux.Response] {
    def apply(req: mux.Request): Future[mux.Response] =
      Future.value(
        mux.Response(Buf.Utf8("Some string"))
      )
  }

  val server = Mux.serve(":8080", service)
  Await.ready(server)
}
