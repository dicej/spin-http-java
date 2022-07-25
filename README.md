## Experimental Spin HTTP app written in Java

This uses [TeaVM](https://github.com/konsoletyper/teavm)'s experimental
WebAssembly backend to convert Java bytecode to WebAssembly, plus a mix of
`@Export`-annotated functions and Wasm post-processing to make it usable as a
[Spin](https://github.com/fermyon/spin) HTTP app.

### Status

The basic "hello, world" app in HelloSpin.java works with the hacks listed
below.  I haven't tried anything more ambitious than that yet, though.

### Building and Running

Prerequisites:

- [Java](https://openjdk.org/install/)
- [Maven](https://maven.apache.org/)
- [Rust](https://rustup.rs/)
- [Spin](https://github.com/fermyon/spin/releases)

First, clone, patch, build, and install TeaVM:

```
git clone https://github.com/konsoletyper/teavm
cd teavm
git checkout ddddfcf2175b7e8e7a9c24877116954835240fb1
git apply <<END
diff --git a/core/src/main/java/org/teavm/backend/wasm/WasmTarget.java b/core/src/main/java/org/teavm/backend/wasm/WasmTarget.java
index 64b9e60a..02c24c81 100644
--- a/core/src/main/java/org/teavm/backend/wasm/WasmTarget.java
+++ b/core/src/main/java/org/teavm/backend/wasm/WasmTarget.java
@@ -421,7 +421,7 @@ public class WasmTarget implements TeaVMTarget, TeaVMWasmHost {
         VirtualTableProvider vtableProvider = createVirtualTableProvider(classes);
         ClassHierarchy hierarchy = new ClassHierarchy(classes);
         TagRegistry tagRegistry = new TagRegistry(classes, hierarchy);
-        BinaryWriter binaryWriter = new BinaryWriter(256);
+        BinaryWriter binaryWriter = new BinaryWriter(64 * 1024);
         NameProvider names = new NameProviderWithSpecialNames(new WasmNameProvider(),
                 controller.getUnprocessedClassSource());
         ClassMetadataRequirements metadataRequirements = new ClassMetadataRequirements(controller.getDependencyInfo());
@@ -489,7 +489,7 @@ public class WasmTarget implements TeaVMTarget, TeaVMWasmHost {

         WasmMemorySegment dataSegment = new WasmMemorySegment();
         dataSegment.setData(binaryWriter.getData());
-        dataSegment.setOffset(256);
+        dataSegment.setOffset(64 * 1024);
         module.getSegments().add(dataSegment);

         renderMemoryLayout(module, binaryWriter.getAddress(), gcIntrinsic);
@@ -1076,4 +1076,4 @@ public class WasmTarget implements TeaVMTarget, TeaVMWasmHost {
             }
         }
     }
-}
\ No newline at end of file
+}
END
mvn clean install
cd ..
```

Then, clone and build and this app:

```
git clone https://github.com/dicej/spin-http-java
cd spin-http-java
spin build && spin up
```

Finally, in another terminal or in a web browser, hit the endpoint:

```
curl -v http://127.0.0.1:3000/hello
```

If all went well, you should see "Hello, Fermyon!".

### Hacks

This currently requires a small patch to TeaVM to reserve room for [canonical
ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)
heap space.  See the `git apply` step above for details.

The Wasm post-processing is handled by a small Rust CLI app located in the
[munge](./munge) directory.  It replaces the "teavm" and "teavmHeapTrace"
imports with stub functions since Spin won't know how to deal with them at
runtime.
