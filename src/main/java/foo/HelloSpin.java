package foo;

import static org.teavm.interop.wasi.Memory.realloc;
import static org.teavm.interop.wasi.Memory.free;
import java.nio.charset.StandardCharsets;
import org.teavm.interop.Address;
import org.teavm.interop.Export;

public class HelloSpin {
  public static void main(String[] args) { }

  private static Address staticResponse = Address.fromInt(0);

  @Export(name = "handle-http-request")
  public static Address handleHttpRequest(
      int method,
      Address uriAddress,
      int uriLength,
      Address headersAddress,
      int headersLength,
      Address paramsAddress,
      int paramsLength,
      int bodyIsSome,
      Address bodyAddress,
      int bodyLength) {
    if (staticResponse.toInt() == 0) {
      byte[] bodyBytes = "Hello, Fermyon!\n".getBytes(StandardCharsets.UTF_8);
      Address body = realloc(Address.fromInt(0), 0, 1, bodyBytes.length);
      for (int i = 0; i < bodyBytes.length; ++i) {
        body.add(i).putByte(bodyBytes[i]);
      }

      Address response = realloc(Address.fromInt(0), 0, 4, 28);
      response.putShort((short) 200);
      response.add(4).putByte((byte) 0);
      response.add(16).putByte((byte) 1);
      response.add(20).putInt(body.toInt());
      response.add(24).putInt(bodyBytes.length);

      staticResponse = response;
    }

    return staticResponse;
  }
}
