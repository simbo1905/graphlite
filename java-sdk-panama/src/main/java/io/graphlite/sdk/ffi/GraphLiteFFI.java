package io.graphlite.sdk.ffi;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.file.Path;
import java.util.List;

import static java.lang.foreign.ValueLayout.ADDRESS;
import static java.lang.foreign.ValueLayout.JAVA_INT;

/**
 * Centralized FFM bindings for GraphLite C FFI.
 * All native interop goes through this class.
 */
public final class GraphLiteFFI {

    private static final String LIB_NAME = "graphlite_ffi";

    private final Arena arena;
    private final SymbolLookup lookup;
    private final Linker linker;

    // Method handles
    private final MethodHandle graphlite_open;
    private final MethodHandle graphlite_create_session;
    private final MethodHandle graphlite_query;
    private final MethodHandle graphlite_close_session;
    private final MethodHandle graphlite_free_string;
    private final MethodHandle graphlite_close;
    private final MethodHandle graphlite_version;

    public GraphLiteFFI(Arena arena) {
        this.arena = arena;
        this.linker = Linker.nativeLinker();
        this.lookup = loadLibrary(arena);
        this.graphlite_open = downcall("graphlite_open");
        this.graphlite_create_session = downcall("graphlite_create_session");
        this.graphlite_query = downcall("graphlite_query");
        this.graphlite_close_session = downcall("graphlite_close_session");
        this.graphlite_free_string = downcall("graphlite_free_string");
        this.graphlite_close = downcall("graphlite_close");
        this.graphlite_version = downcall("graphlite_version");
    }

    private SymbolLookup loadLibrary(Arena arena) {
        String envLib = System.getenv("GRAPHLITE_FFI_LIB");
        if (envLib != null && !envLib.isBlank()) {
            return SymbolLookup.libraryLookup(Path.of(envLib), arena);
        }

        String libFileName = platformLibraryName();
        Path libPath = findLibrary(libFileName);
        return SymbolLookup.libraryLookup(libPath, arena);
    }

    private static String platformLibraryName() {
        String os = System.getProperty("os.name").toLowerCase();
        if (os.contains("mac")) {
            return "libgraphlite_ffi.dylib";
        }
        if (os.contains("win")) {
            return "graphlite_ffi.dll";
        }
        return "libgraphlite_ffi.so";
    }

    private static Path findLibrary(String libFileName) {
        Path cwd = Path.of("").toAbsolutePath();
        // Build candidate list: cwd-relative, then walk up for multi-module (examples/java/sdk-panama -> repo root)
        List<Path> candidates = new java.util.ArrayList<>();
        candidates.add(cwd.resolve("target/release").resolve(libFileName));
        candidates.add(cwd.resolve("target/debug").resolve(libFileName));
        candidates.add(cwd.resolve(libFileName));
        Path p = cwd;
        for (int i = 0; i < 5 && p != null; i++) {
            candidates.add(p.resolve("target/release").resolve(libFileName));
            candidates.add(p.resolve("target/debug").resolve(libFileName));
            p = p.getParent();
        }
        candidates.add(Path.of("/usr/local/lib").resolve(libFileName));
        candidates.add(Path.of("/usr/lib").resolve(libFileName));

        for (Path cand : candidates) {
            try {
                if (java.nio.file.Files.exists(cand)) {
                    return cand.toAbsolutePath();
                }
            } catch (Exception ignored) {
            }
        }

        throw new UnsatisfiedLinkError(
            "Could not find GraphLite library (" + libFileName + "). " +
            "Set GRAPHLITE_FFI_LIB to the full path, or build: cargo build --release -p graphlite-ffi"
        );
    }

    private MethodHandle downcall(String name) {
        MemorySegment symbol = lookup.find(name).orElseThrow(() ->
            new UnsatisfiedLinkError("Symbol not found: " + name));
        return switch (name) {
            case "graphlite_open" -> linker.downcallHandle(symbol, FunctionDescriptor.of(ADDRESS, ADDRESS, ADDRESS));
            case "graphlite_create_session" -> linker.downcallHandle(symbol, FunctionDescriptor.of(ADDRESS, ADDRESS, ADDRESS, ADDRESS));
            case "graphlite_query" -> linker.downcallHandle(symbol, FunctionDescriptor.of(ADDRESS, ADDRESS, ADDRESS, ADDRESS, ADDRESS));
            case "graphlite_close_session" -> linker.downcallHandle(symbol, FunctionDescriptor.of(JAVA_INT, ADDRESS, ADDRESS, ADDRESS));
            case "graphlite_free_string" -> linker.downcallHandle(symbol, FunctionDescriptor.ofVoid(ADDRESS));
            case "graphlite_close" -> linker.downcallHandle(symbol, FunctionDescriptor.ofVoid(ADDRESS));
            case "graphlite_version" -> linker.downcallHandle(symbol, FunctionDescriptor.of(ADDRESS));
            default -> throw new IllegalArgumentException("Unknown symbol: " + name);
        };
    }

    /** Open database; returns null on error, check errorOut. Caller must call graphlite_close. */
    public MemorySegment graphlite_open(String path, MemorySegment errorOut) throws Throwable {
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment pathSeg = callArena.allocateUtf8String(path);
            return (MemorySegment) graphlite_open.invoke(pathSeg, errorOut);
        }
    }

    /** Create session; returns null on error. Caller must call graphlite_free_string on result. */
    public MemorySegment graphlite_create_session(MemorySegment db, String username, MemorySegment errorOut) throws Throwable {
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment userSeg = callArena.allocateUtf8String(username);
            return (MemorySegment) graphlite_create_session.invoke(db, userSeg, errorOut);
        }
    }

    /** Execute query; returns JSON string or null on error. Caller must call graphlite_free_string on result. */
    public MemorySegment graphlite_query(MemorySegment db, String sessionId, String query, MemorySegment errorOut) throws Throwable {
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment sessionSeg = callArena.allocateUtf8String(sessionId);
            MemorySegment querySeg = callArena.allocateUtf8String(query);
            return (MemorySegment) graphlite_query.invoke(db, sessionSeg, querySeg, errorOut);
        }
    }

    /** Close session; returns error code. */
    public int graphlite_close_session(MemorySegment db, String sessionId, MemorySegment errorOut) throws Throwable {
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment sessionSeg = callArena.allocateUtf8String(sessionId);
            return (int) graphlite_close_session.invoke(db, sessionSeg, errorOut);
        }
    }

    /** Free string returned by FFI. */
    public void graphlite_free_string(MemorySegment str) throws Throwable {
        if (str != null && !str.equals(MemorySegment.NULL)) {
            graphlite_free_string.invoke(str);
        }
    }

    /** Close database. */
    public void graphlite_close(MemorySegment db) throws Throwable {
        if (db != null && !db.equals(MemorySegment.NULL)) {
            graphlite_close.invoke(db);
        }
    }

    /** Get version string (static, do NOT free). */
    public String graphlite_version() throws Throwable {
        MemorySegment versionPtr = (MemorySegment) graphlite_version.invoke();
        if (versionPtr == null || versionPtr.equals(MemorySegment.NULL)) {
            return "unknown";
        }
        return versionPtr.getUtf8String(0);
    }

    /** Read C string from segment and return Java String. */
    public static String readCString(MemorySegment seg) {
        if (seg == null || seg.equals(MemorySegment.NULL)) {
            return null;
        }
        return seg.getUtf8String(0);
    }
}
