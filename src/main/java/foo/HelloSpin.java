package foo;

import org.teavm.interop.Address;
import org.teavm.interop.Export;

public class HelloSpin {
  public static void main(String[] args) {}

  private static Address RESPONSE = Address.fromInt(0);
  private static Address NEXT = Address.fromInt(4);
  // This is intended to match the Java heap start address specified as the
  // argument to the `BinaryWriter` constructor in
  // org.teavm.backend.wasm.WasmTarget.  You'll need to patch TeaVM to make
  // these match since upstream TeaVM uses a value of 256 as of this writing.
  private static Address LIMIT = Address.fromInt(64 * 1024);

  // Implements a trivial bump allocator which does not support freeing allocations
  @Export(name = "canonical_abi_realloc")
  public static Address realloc(Address oldAddress, int oldSize, int align, int newSize) {
    if (oldAddress.toInt() == 0) {
      Address candidate = Address.align(NEXT, align);
      Address next = candidate.add(newSize);
      if (next.toInt() > LIMIT.toInt()) {
        throw new RuntimeException();
      } else {
        NEXT = next;
        return candidate;
      }
    } else if (newSize <= oldSize) {
      return oldAddress;
    } else {
      throw new RuntimeException();
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
      String bodyString = "Hello, Fermyon!\n";
      // TODO: Obviously we should be using `String.getBytes()` here instead of
      // just casting the chars to bytes, but as of this writing that causes
      // weird heap and/or stack corruption which I haven't been able to debug
      // yet.
      Address body = realloc(Address.fromInt(0), 0, 1, bodyString.length());
      for (int i = 0; i < bodyString.length(); ++i) {
        body.add(i).putByte((byte) bodyString.charAt(i));
      }

      Address response = realloc(Address.fromInt(0), 0, 4, 28);
      response.putShort((short) 200);
      response.add(4).putByte((byte) 0);
      response.add(16).putByte((byte) 1);
      response.add(20).putInt(body.toInt());
      response.add(24).putInt(bodyString.length());

      RESPONSE = response;
    }

    return RESPONSE;
  }
}
