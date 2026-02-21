package io.graphlite.sdk.ffi;

import java.lang.foreign.AddressLayout;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.StructLayout;
import java.lang.foreign.ValueLayout;
import java.nio.ByteOrder;

/**
 * Shared memory layouts for GraphLite FFI bindings.
 */
public final class Layouts {
    private Layouts() {
    }

    /**
     * GraphLiteErrorCode is represented as a C int in the FFI.
     */
    public static final ValueLayout.OfInt ERROR_CODE = ValueLayout.JAVA_INT.withOrder(ByteOrder.nativeOrder());

    /**
     * Generic C pointer layout.
     */
    public static final AddressLayout C_POINTER = ValueLayout.ADDRESS;

    /**
     * C char* layout (null-terminated UTF-8 strings).
     */
    public static final AddressLayout C_STRING = ValueLayout.ADDRESS.withName("char*");

    /**
     * Opaque GraphLiteDB* handle layout.
     */
    public static final AddressLayout GRAPH_LITE_DB_HANDLE = ValueLayout.ADDRESS.withName("GraphLiteDB*");

    /**
     * Opaque placeholder for GraphLiteDB struct metadata.
     * The real struct is owned by Rust and treated as opaque in Java.
     */
    public static final StructLayout GRAPH_LITE_DB_OPAQUE = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("_opaque")
    ).withName("GraphLiteDB");
}
