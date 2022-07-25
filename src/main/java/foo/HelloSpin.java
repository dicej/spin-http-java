package foo;

import java.nio.charset.StandardCharsets;
import org.teavm.interop.Address;
import org.teavm.interop.Export;

public class HelloSpin {
  public static void main(String[] args) { }

  private static Address RESPONSE = Address.fromInt(0);
  private static Address NEXT = Address.fromInt(4);
  // This is intended to match certain values in
  // org.teavm.backend.wasm.WasmTarget.  You'll need to patch TeaVM to make
  // these match since upstream TeaVM uses a value of 256 as of this writing.
  // See README.md for details.
  private static Address LIMIT = Address.fromInt(64 * 1024);

  // Implements a trivial bump allocator which does not support freeing allocations
  @Export(name = "canonical_abi_realloc")
  public static Address realloc(Address oldAddress, int oldSize, int align, int newSize) {
    if (oldAddress.toInt() != 0 && newSize <= oldSize) {
      return oldAddress;
    }

    Address candidate = Address.align(NEXT, align);
    Address next = candidate.add(newSize);
    if (next.toInt() > LIMIT.toInt()) {
      throw new OutOfMemoryError();
    } else {
      NEXT = next;
      return candidate;
    }
  }

  // Does nothing
  @Export(name = "canonical_abi_free")
  public static void free(Address address, int size, int align) {}

  // Implements the spin-http.wit handle-http-request function
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
    if (RESPONSE.toInt() == 0) {
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

      RESPONSE = response;
    }

    return RESPONSE;
  }
}
