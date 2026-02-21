package io.graphlite.sdk.ffi;

import java.lang.foreign.ValueLayout;

/**
 * Memory layouts for GraphLite FFI structs.
 * GraphLiteDB is opaque; we only pass pointers. Error codes are int-sized.
 */
public final class Layouts {
    private Layouts() {}

    /** C int for GraphLiteErrorCode output parameter */
    public static final ValueLayout.OfInt C_INT = ValueLayout.JAVA_INT;
}
