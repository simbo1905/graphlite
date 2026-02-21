package io.graphlite.sdk.ffi;

import io.graphlite.sdk.Errors;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.invoke.MethodHandle;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;

/**
 * Centralized GraphLite foreign interop boundary using Java 25 FFM API.
 *
 * <p>All native calls in the high-level SDK go through this class.</p>
 */
public final class GraphLiteFFI {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final String LIB_ENV_OVERRIDE = "GRAPHLITE_FFI_LIB";
    private static final String DEFAULT_LIB_NAME = "graphlite_ffi";

    private final Arena symbolArena;
    private final SymbolLookup lookup;

    private final MethodHandle graphliteOpen;
    private final MethodHandle graphliteCreateSession;
    private final MethodHandle graphliteQuery;
    private final MethodHandle graphliteCloseSession;
    private final MethodHandle graphliteFreeString;
    private final MethodHandle graphliteClose;
    private final MethodHandle graphliteVersion;

    private GraphLiteFFI() {
        validatePlatform();
        this.symbolArena = Arena.ofShared();
        this.lookup = resolveLookup(symbolArena);

        this.graphliteOpen = downcall(
            "graphlite_open",
            FunctionDescriptor.of(Layouts.GRAPH_LITE_DB_HANDLE, Layouts.C_STRING, Layouts.C_POINTER)
        );
        this.graphliteCreateSession = downcall(
            "graphlite_create_session",
            FunctionDescriptor.of(Layouts.C_STRING, Layouts.GRAPH_LITE_DB_HANDLE, Layouts.C_STRING, Layouts.C_POINTER)
        );
        this.graphliteQuery = downcall(
            "graphlite_query",
            FunctionDescriptor.of(
                Layouts.C_STRING,
                Layouts.GRAPH_LITE_DB_HANDLE,
                Layouts.C_STRING,
                Layouts.C_STRING,
                Layouts.C_POINTER
            )
        );
        this.graphliteCloseSession = downcall(
            "graphlite_close_session",
            FunctionDescriptor.of(
                Layouts.ERROR_CODE,
                Layouts.GRAPH_LITE_DB_HANDLE,
                Layouts.C_STRING,
                Layouts.C_POINTER
            )
        );
        this.graphliteFreeString = downcall(
            "graphlite_free_string",
            FunctionDescriptor.ofVoid(Layouts.C_STRING)
        );
        this.graphliteClose = downcall(
            "graphlite_close",
            FunctionDescriptor.ofVoid(Layouts.GRAPH_LITE_DB_HANDLE)
        );
        this.graphliteVersion = downcall(
            "graphlite_version",
            FunctionDescriptor.of(Layouts.C_STRING)
        );
    }

    public static GraphLiteFFI shared() {
        return Holder.INSTANCE;
    }

    public MemorySegment openDatabase(String path) {
        Objects.requireNonNull(path, "path must not be null");
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment cPath = callArena.allocateFrom(path);
            MemorySegment errorOut = callArena.allocate(Layouts.ERROR_CODE);
            errorOut.set(Layouts.ERROR_CODE, 0L, Errors.ErrorCode.SUCCESS.code());

            MemorySegment dbHandle = (MemorySegment) graphliteOpen.invoke(cPath, errorOut);
            int errorCode = errorOut.get(Layouts.ERROR_CODE, 0L);
            if (isNullPointer(dbHandle)) {
                throw Errors.connection(errorCode, "Failed to open database at " + path);
            }
            return dbHandle;
        } catch (Errors.GraphLiteException e) {
            throw e;
        } catch (Throwable t) {
            throw Errors.connection(
                Errors.ErrorCode.UNKNOWN.code(),
                "FFI invocation failed while opening database",
                t
            );
        }
    }

    public String createSession(MemorySegment dbHandle, String username) {
        Objects.requireNonNull(dbHandle, "dbHandle must not be null");
        Objects.requireNonNull(username, "username must not be null");
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment cUsername = callArena.allocateFrom(username);
            MemorySegment errorOut = callArena.allocate(Layouts.ERROR_CODE);
            errorOut.set(Layouts.ERROR_CODE, 0L, Errors.ErrorCode.SUCCESS.code());

            MemorySegment sessionPtr = (MemorySegment) graphliteCreateSession.invoke(dbHandle, cUsername, errorOut);
            int errorCode = errorOut.get(Layouts.ERROR_CODE, 0L);
            if (isNullPointer(sessionPtr)) {
                throw Errors.session(errorCode, "Failed to create session for user '" + username + "'");
            }
            return copyOwnedCString(sessionPtr);
        } catch (Errors.GraphLiteException e) {
            throw e;
        } catch (Throwable t) {
            throw Errors.session(
                Errors.ErrorCode.UNKNOWN.code(),
                "FFI invocation failed while creating session",
                t
            );
        }
    }

    public String query(MemorySegment dbHandle, String sessionId, String query) {
        Objects.requireNonNull(dbHandle, "dbHandle must not be null");
        Objects.requireNonNull(sessionId, "sessionId must not be null");
        Objects.requireNonNull(query, "query must not be null");
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment cSessionId = callArena.allocateFrom(sessionId);
            MemorySegment cQuery = callArena.allocateFrom(query);
            MemorySegment errorOut = callArena.allocate(Layouts.ERROR_CODE);
            errorOut.set(Layouts.ERROR_CODE, 0L, Errors.ErrorCode.SUCCESS.code());

            MemorySegment resultPtr = (MemorySegment) graphliteQuery.invoke(dbHandle, cSessionId, cQuery, errorOut);
            int errorCode = errorOut.get(Layouts.ERROR_CODE, 0L);
            if (isNullPointer(resultPtr)) {
                throw Errors.query(errorCode, "Query failed: " + summarize(query));
            }
            return copyOwnedCString(resultPtr);
        } catch (Errors.GraphLiteException e) {
            throw e;
        } catch (Throwable t) {
            throw Errors.query(
                Errors.ErrorCode.UNKNOWN.code(),
                "FFI invocation failed while executing query",
                t
            );
        }
    }

    public void closeSession(MemorySegment dbHandle, String sessionId) {
        Objects.requireNonNull(dbHandle, "dbHandle must not be null");
        Objects.requireNonNull(sessionId, "sessionId must not be null");
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment cSessionId = callArena.allocateFrom(sessionId);
            MemorySegment errorOut = callArena.allocate(Layouts.ERROR_CODE);
            errorOut.set(Layouts.ERROR_CODE, 0L, Errors.ErrorCode.SUCCESS.code());

            int returnCode = (int) graphliteCloseSession.invoke(dbHandle, cSessionId, errorOut);
            int outCode = errorOut.get(Layouts.ERROR_CODE, 0L);
            int effectiveCode = returnCode != Errors.ErrorCode.SUCCESS.code() ? returnCode : outCode;

            if (effectiveCode != Errors.ErrorCode.SUCCESS.code()) {
                throw Errors.session(effectiveCode, "Failed to close session '" + sessionId + "'");
            }
        } catch (Errors.GraphLiteException e) {
            throw e;
        } catch (Throwable t) {
            throw Errors.session(
                Errors.ErrorCode.UNKNOWN.code(),
                "FFI invocation failed while closing session",
                t
            );
        }
    }

    public void closeDatabase(MemorySegment dbHandle) {
        if (isNullPointer(dbHandle)) {
            return;
        }
        try {
            graphliteClose.invoke(dbHandle);
        } catch (Throwable t) {
            throw Errors.connection(
                Errors.ErrorCode.UNKNOWN.code(),
                "FFI invocation failed while closing database",
                t
            );
        }
    }

    public String version() {
        try {
            MemorySegment versionPtr = (MemorySegment) graphliteVersion.invoke();
            if (isNullPointer(versionPtr)) {
                return "unknown";
            }
            return readCString(versionPtr);
        } catch (Throwable t) {
            throw new Errors.NativeLibraryException("Failed to read GraphLite version", t);
        }
    }

    private MethodHandle downcall(String symbol, FunctionDescriptor descriptor) {
        MemorySegment symbolAddress = lookup.find(symbol)
            .orElseThrow(() -> new Errors.NativeLibraryException(
                "Native symbol not found: " + symbol + ". Verify libgraphlite_ffi exports this function."
            ));
        return LINKER.downcallHandle(symbolAddress, descriptor);
    }

    private SymbolLookup resolveLookup(Arena arena) {
        String override = System.getenv(LIB_ENV_OVERRIDE);
        if (override != null && !override.isBlank()) {
            return loadLookupFromToken(override.trim(), arena);
        }

        List<String> errors = new ArrayList<>();
        try {
            // First try by bare library name, e.g. libgraphlite_ffi.so / .dylib.
            return SymbolLookup.libraryLookup(DEFAULT_LIB_NAME, arena);
        } catch (Throwable t) {
            errors.add(DEFAULT_LIB_NAME + ": " + oneLine(t));
        }

        for (Path candidate : defaultLibraryCandidates(platformLibraryFileName())) {
            if (!Files.exists(candidate)) {
                continue;
            }
            try {
                return SymbolLookup.libraryLookup(candidate, arena);
            } catch (Throwable t) {
                errors.add(candidate + ": " + oneLine(t));
            }
        }

        throw new Errors.NativeLibraryException(
            "Could not locate GraphLite FFI library. "
                + "Set " + LIB_ENV_OVERRIDE + " to an absolute path to libgraphlite_ffi. "
                + "Lookup attempts: " + String.join(" | ", errors)
        );
    }

    private SymbolLookup loadLookupFromToken(String token, Arena arena) {
        try {
            if (looksLikePath(token)) {
                Path path = Path.of(token).toAbsolutePath().normalize();
                if (!Files.exists(path)) {
                    throw new Errors.NativeLibraryException(
                        LIB_ENV_OVERRIDE + " points to a missing file: " + path
                    );
                }
                return SymbolLookup.libraryLookup(path, arena);
            }
            return SymbolLookup.libraryLookup(token, arena);
        } catch (InvalidPathException e) {
            throw new Errors.NativeLibraryException(
                "Invalid path supplied in " + LIB_ENV_OVERRIDE + ": " + token,
                e
            );
        } catch (Errors.NativeLibraryException e) {
            throw e;
        } catch (Throwable t) {
            throw new Errors.NativeLibraryException(
                "Failed to load native library from " + LIB_ENV_OVERRIDE + "=" + token,
                t
            );
        }
    }

    private static List<Path> defaultLibraryCandidates(String libraryFileName) {
        Path cwd = Path.of("").toAbsolutePath().normalize();
        Set<Path> candidates = new LinkedHashSet<>();

        candidates.add(cwd.resolve(libraryFileName));
        candidates.add(cwd.resolve("target").resolve("release").resolve(libraryFileName));
        candidates.add(cwd.resolve("target").resolve("debug").resolve(libraryFileName));
        candidates.add(cwd.resolve("..").resolve("target").resolve("release").resolve(libraryFileName).normalize());
        candidates.add(cwd.resolve("..").resolve("target").resolve("debug").resolve(libraryFileName).normalize());
        candidates.add(cwd.resolve("..").resolve("..").resolve("target").resolve("release").resolve(libraryFileName).normalize());
        candidates.add(cwd.resolve("..").resolve("..").resolve("target").resolve("debug").resolve(libraryFileName).normalize());
        candidates.add(cwd.resolve("..").resolve("..").resolve("..").resolve("target").resolve("release").resolve(libraryFileName).normalize());
        candidates.add(cwd.resolve("..").resolve("..").resolve("..").resolve("target").resolve("debug").resolve(libraryFileName).normalize());

        String libraryPath = System.getProperty("java.library.path", "");
        if (!libraryPath.isBlank()) {
            String[] entries = libraryPath.split(java.io.File.pathSeparator);
            for (String entry : entries) {
                if (entry == null || entry.isBlank()) {
                    continue;
                }
                try {
                    candidates.add(Path.of(entry).resolve(libraryFileName).normalize());
                } catch (InvalidPathException ignored) {
                    // Ignore malformed entries and continue.
                }
            }
        }

        return List.copyOf(candidates);
    }

    private static String platformLibraryFileName() {
        String os = normalizedOsName();
        if (os.contains("mac")) {
            return "libgraphlite_ffi.dylib";
        }
        if (os.contains("win")) {
            return "graphlite_ffi.dll";
        }
        return "libgraphlite_ffi.so";
    }

    private static boolean looksLikePath(String token) {
        return token.contains("/")
            || token.contains("\\")
            || token.endsWith(".so")
            || token.endsWith(".dylib")
            || token.endsWith(".dll")
            || token.startsWith(".");
    }

    private static String readCString(MemorySegment pointer) {
        return pointer.reinterpret(Long.MAX_VALUE).getString(0L);
    }

    private String copyOwnedCString(MemorySegment pointer) {
        try {
            return readCString(pointer);
        } finally {
            freeOwnedCString(pointer);
        }
    }

    private void freeOwnedCString(MemorySegment pointer) {
        if (isNullPointer(pointer)) {
            return;
        }
        try {
            graphliteFreeString.invoke(pointer);
        } catch (Throwable t) {
            throw new Errors.NativeLibraryException("Failed to free native string", t);
        }
    }

    private static boolean isNullPointer(MemorySegment segment) {
        return segment == null || MemorySegment.NULL.equals(segment);
    }

    private static String summarize(String query) {
        String trimmed = query.strip();
        return trimmed.length() <= 140 ? trimmed : trimmed.substring(0, 140) + "...";
    }

    private static String oneLine(Throwable throwable) {
        String message = throwable.getMessage();
        if (message == null) {
            return throwable.getClass().getSimpleName();
        }
        return throwable.getClass().getSimpleName() + ": " + message.replace('\n', ' ').trim();
    }

    private static String normalizedOsName() {
        return System.getProperty("os.name", "unknown").toLowerCase(Locale.ROOT);
    }

    private static void validatePlatform() {
        if (normalizedOsName().contains("win")) {
            throw new Errors.NativeLibraryException(
                "Windows is not supported by this Panama demo yet. "
                    + "Use Linux/macOS (or WSL) and point " + LIB_ENV_OVERRIDE + " to libgraphlite_ffi."
            );
        }
    }

    private static final class Holder {
        private static final GraphLiteFFI INSTANCE = new GraphLiteFFI();

        private Holder() {
        }
    }
}
