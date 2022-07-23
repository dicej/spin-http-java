package foo;

import org.teavm.interop.Address;
import org.teavm.interop.Export;

public class HelloSpin {
  public static void main(String[] args) {}

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
    int[] response = new int[7];
    byte[] body = "Hello, Spin -- from Java!".getBytes();
    response[0] = 400;
    response[4] = 1;
    response[5] = Address.ofData(body).toInt();
    response[6] = body.length;
    return Address.ofData(body);
  }
}
