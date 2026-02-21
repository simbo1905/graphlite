package io.graphlite.sdk.ffi;

import java.lang.foreign.MemoryLayout;
import java.lang.foreign.ValueLayout;

/**
 * Memory layouts corresponding to native structs used by the graphlite-ffi C API.
 * <p>
 * The FFI functions mostly operate on opaque pointers ({@code GraphLiteDB*}) and
 * C strings ({@code const char*}). This class centralizes layout definitions so
 * callers don't have to hard-code them.
 */
public final class Layouts {

    private Layouts() {}

    /** Layout for the native error-code enum ({@code GraphLiteErrorCode}, C int). */
    public static final ValueLayout.OfInt ERROR_CODE = ValueLayout.JAVA_INT;

    /** Layout for a native pointer (used for opaque handles and C strings). */
    public static final ValueLayout POINTER = ValueLayout.ADDRESS;

    /**
     * Layout for a single {@code GraphLiteErrorCode} output parameter.
     * Allocated as a one-element int segment that the native side writes to.
     */
    public static final MemoryLayout ERROR_OUT = MemoryLayout.structLayout(
            ERROR_CODE.withName("code")
    ).withName("ErrorOut");
}
