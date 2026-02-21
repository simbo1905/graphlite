package io.graphlite.sdk.ffi;

import io.graphlite.sdk.Errors;
import io.graphlite.sdk.Errors.ErrorCode;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Centralizes all Foreign Function &amp; Memory API (Panama FFM) bindings to the
 * {@code libgraphlite_ffi} shared library.
 * <p>
 * Every native call is a thin wrapper around a {@link MethodHandle} downcall
 * obtained from {@link Linker#downcallHandle}.  Callers should prefer the
 * high-level {@link io.graphlite.sdk.GraphLite} / {@link io.graphlite.sdk.Session}
 * classes instead of invoking these handles directly.
 * <p>
 * <b>Library resolution order:</b>
 * <ol>
 *   <li>Environment variable {@code GRAPHLITE_FFI_LIB} &ndash; full path to the shared library.</li>
 *   <li>System property {@code graphlite.ffi.lib} &ndash; full path to the shared library.</li>
 *   <li>Platform-default search paths (respects {@code LD_LIBRARY_PATH} / {@code DYLD_LIBRARY_PATH} / {@code PATH}).</li>
 * </ol>
 */
public final class GraphLiteFFI {

    private GraphLiteFFI() {}

    private static final Linker LINKER = Linker.nativeLinker();

    // --- Method handles (initialized lazily on first access) ---

    private static volatile SymbolLookup lookup;
    private static volatile Arena sharedArena;

    private static MethodHandle MH_open;
    private static MethodHandle MH_createSession;
    private static MethodHandle MH_query;
    private static MethodHandle MH_closeSession;
    private static MethodHandle MH_freeString;
    private static MethodHandle MH_close;
    private static MethodHandle MH_version;

    // --- Public initialization ---

    /**
     * Initialize the FFI layer by loading the native library.
     * This is idempotent; subsequent calls are no-ops.
     *
     * @throws Errors.LibraryLoadException if the library cannot be found/loaded.
     */
    public static synchronized void init() {
        if (lookup != null) return;
        try {
            sharedArena = Arena.ofShared();
            lookup = resolveLibrary(sharedArena);
            bindAll();
        } catch (Errors.LibraryLoadException e) {
            throw e;
        } catch (Exception e) {
            throw new Errors.LibraryLoadException("Failed to initialize GraphLite FFI", e);
        }
    }

    // --- Downcall wrappers -----------------------------------------------

    /**
     * {@code GraphLiteDB* graphlite_open(const char* path, GraphLiteErrorCode* error_out)}
     *
     * @return native handle (opaque pointer) to the database, or {@link MemorySegment#NULL} on error.
     */
    public static MemorySegment open(MemorySegment path, MemorySegment errorOut) {
        try {
            return (MemorySegment) MH_open.invokeExact(path, errorOut);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code char* graphlite_create_session(GraphLiteDB* db, const char* username, GraphLiteErrorCode* error_out)}
     */
    public static MemorySegment createSession(MemorySegment db, MemorySegment username, MemorySegment errorOut) {
        try {
            return (MemorySegment) MH_createSession.invokeExact(db, username, errorOut);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code char* graphlite_query(GraphLiteDB* db, const char* session_id, const char* query, GraphLiteErrorCode* error_out)}
     */
    public static MemorySegment query(MemorySegment db, MemorySegment sessionId, MemorySegment query, MemorySegment errorOut) {
        try {
            return (MemorySegment) MH_query.invokeExact(db, sessionId, query, errorOut);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code GraphLiteErrorCode graphlite_close_session(GraphLiteDB* db, const char* session_id, GraphLiteErrorCode* error_out)}
     */
    public static int closeSession(MemorySegment db, MemorySegment sessionId, MemorySegment errorOut) {
        try {
            return (int) MH_closeSession.invokeExact(db, sessionId, errorOut);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code void graphlite_free_string(char* s)}
     */
    public static void freeString(MemorySegment s) {
        try {
            MH_freeString.invokeExact(s);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code void graphlite_close(GraphLiteDB* db)}
     */
    public static void close(MemorySegment db) {
        try {
            MH_close.invokeExact(db);
        } catch (Throwable t) { throw rethrow(t); }
    }

    /**
     * {@code const char* graphlite_version()}
     * Returns a pointer to a static string &mdash; must <b>not</b> be freed.
     */
    public static MemorySegment version() {
        try {
            return (MemorySegment) MH_version.invokeExact();
        } catch (Throwable t) { throw rethrow(t); }
    }

    // --- Utility helpers for callers --------------------------------------

    /** Allocate a null-terminated UTF-8 C string inside the given arena. */
    public static MemorySegment toCString(String s, Arena arena) {
        return arena.allocateUtf8String(s);
    }

    /** Read a null-terminated UTF-8 C string from a native pointer. */
    public static String fromCString(MemorySegment ptr) {
        return ptr.reinterpret(Long.MAX_VALUE).getUtf8String(0);
    }

    /** Allocate a single-int segment for the error-out parameter. */
    public static MemorySegment allocErrorOut(Arena arena) {
        MemorySegment seg = arena.allocate(Layouts.ERROR_CODE);
        seg.set(Layouts.ERROR_CODE, 0, 0);
        return seg;
    }

    /** Read the error code written by the native side. */
    public static ErrorCode readErrorCode(MemorySegment errorOut) {
        return ErrorCode.fromNative(errorOut.get(Layouts.ERROR_CODE, 0));
    }

    // --- Internal ---------------------------------------------------------

    private static SymbolLookup resolveLibrary(Arena arena) {
        String envPath = System.getenv("GRAPHLITE_FFI_LIB");
        if (envPath != null && !envPath.isBlank()) {
            return loadFrom(Path.of(envPath), arena);
        }
        String sysProp = System.getProperty("graphlite.ffi.lib");
        if (sysProp != null && !sysProp.isBlank()) {
            return loadFrom(Path.of(sysProp), arena);
        }

        String libFile = platformLibraryName();
        String[] candidates = {
            "target/release/" + libFile,
            "../target/release/" + libFile,
            "../../target/release/" + libFile,
            "target/debug/" + libFile,
            "../target/debug/" + libFile,
            "../../target/debug/" + libFile,
        };
        for (String candidate : candidates) {
            Path p = Path.of(candidate).toAbsolutePath().normalize();
            if (Files.isRegularFile(p)) {
                return loadFrom(p, arena);
            }
        }

        try {
            return SymbolLookup.libraryLookup(System.mapLibraryName("graphlite_ffi"), arena);
        } catch (IllegalArgumentException e) {
            throw new Errors.LibraryLoadException(
                "Could not locate libgraphlite_ffi. Set GRAPHLITE_FFI_LIB or " +
                "LD_LIBRARY_PATH/DYLD_LIBRARY_PATH to the directory containing the library. " +
                "Build with: cargo build --release -p graphlite-ffi", e);
        }
    }

    private static SymbolLookup loadFrom(Path path, Arena arena) {
        if (!Files.isRegularFile(path)) {
            throw new Errors.LibraryLoadException(
                "Library not found at " + path.toAbsolutePath(), null);
        }
        return SymbolLookup.libraryLookup(path, arena);
    }

    private static String platformLibraryName() {
        String os = System.getProperty("os.name", "").toLowerCase();
        if (os.contains("mac") || os.contains("darwin")) {
            return "libgraphlite_ffi.dylib";
        } else if (os.contains("win")) {
            return "graphlite_ffi.dll";
        }
        return "libgraphlite_ffi.so";
    }

    private static MemorySegment findSymbol(String name) {
        return lookup.find(name).orElseThrow(() ->
            new Errors.LibraryLoadException("Symbol not found: " + name, null));
    }

    private static void bindAll() {
        // GraphLiteDB* graphlite_open(const char*, GraphLiteErrorCode*)
        MH_open = LINKER.downcallHandle(
            findSymbol("graphlite_open"),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
        );
        // char* graphlite_create_session(GraphLiteDB*, const char*, GraphLiteErrorCode*)
        MH_createSession = LINKER.downcallHandle(
            findSymbol("graphlite_create_session"),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
        );
        // char* graphlite_query(GraphLiteDB*, const char*, const char*, GraphLiteErrorCode*)
        MH_query = LINKER.downcallHandle(
            findSymbol("graphlite_query"),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
        );
        // GraphLiteErrorCode graphlite_close_session(GraphLiteDB*, const char*, GraphLiteErrorCode*)
        MH_closeSession = LINKER.downcallHandle(
            findSymbol("graphlite_close_session"),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
        );
        // void graphlite_free_string(char*)
        MH_freeString = LINKER.downcallHandle(
            findSymbol("graphlite_free_string"),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );
        // void graphlite_close(GraphLiteDB*)
        MH_close = LINKER.downcallHandle(
            findSymbol("graphlite_close"),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );
        // const char* graphlite_version()
        MH_version = LINKER.downcallHandle(
            findSymbol("graphlite_version"),
            FunctionDescriptor.of(ValueLayout.ADDRESS)
        );
    }

    @SuppressWarnings("unchecked")
    private static RuntimeException rethrow(Throwable t) {
        if (t instanceof RuntimeException re) throw re;
        if (t instanceof Error e) throw e;
        throw new RuntimeException(t);
    }
}
