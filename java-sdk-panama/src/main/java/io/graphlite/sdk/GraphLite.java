package io.graphlite.sdk;

import io.graphlite.sdk.ffi.GraphLiteFFI;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;

/**
 * High-level GraphLite database entry point.
 * Mirrors Python SDK: GraphLite.open(path) -> GraphLite, AutoCloseable.
 */
public final class GraphLite implements AutoCloseable {

    private final MemorySegment dbHandle;
    private final GraphLiteFFI ffi;
    private final Arena arena;
    private boolean closed;

    private GraphLite(MemorySegment dbHandle, GraphLiteFFI ffi, Arena arena) {
        this.dbHandle = dbHandle;
        this.ffi = ffi;
        this.arena = arena;
        this.closed = false;
    }

    /**
     * Open a GraphLite database at the given path.
     *
     * @param path Path to the database directory
     * @return GraphLite instance (must be closed)
     * @throws ConnectionException if database cannot be opened
     */
    public static GraphLite open(String path) {
        Arena arena = Arena.ofConfined();
        GraphLiteFFI ffi = new GraphLiteFFI(arena);
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment errorOut = callArena.allocate(ValueLayout.JAVA_INT);
            MemorySegment db = ffi.graphlite_open(path, errorOut);
            if (db == null || db.equals(MemorySegment.NULL)) {
                int code = errorOut.get(ValueLayout.JAVA_INT, 0);
                arena.close();
                throw new ConnectionException(code, "Failed to open database at " + path);
            }
            return new GraphLite(db, ffi, arena);
        } catch (ConnectionException e) {
            arena.close();
            throw e;
        } catch (Throwable t) {
            arena.close();
            throw new ConnectionException(GraphLiteException.ErrorCode.PANIC_ERROR.getCode(),
                "Failed to open database: " + t.getMessage(), t);
        }
    }

    /**
     * Create a new session for the given user.
     *
     * @param username Username for the session
     * @return Session instance (AutoCloseable)
     * @throws SessionException if session creation fails
     */
    public Session session(String username) {
        ensureOpen();
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment errorOut = callArena.allocate(ValueLayout.JAVA_INT);
            MemorySegment sessionIdPtr = ffi.graphlite_create_session(dbHandle, username, errorOut);
            if (sessionIdPtr == null || sessionIdPtr.equals(MemorySegment.NULL)) {
                int code = errorOut.get(ValueLayout.JAVA_INT, 0);
                throw new SessionException(code, "Failed to create session for user '" + username + "'");
            }
            String sessionId = GraphLiteFFI.readCString(sessionIdPtr);
            ffi.graphlite_free_string(sessionIdPtr);
            return new Session(sessionId, this);
        } catch (SessionException e) {
            throw e;
        } catch (Throwable t) {
            throw new SessionException(GraphLiteException.ErrorCode.PANIC_ERROR.getCode(),
                "Session creation failed: " + t.getMessage(), t);
        }
    }

    MemorySegment dbHandle() {
        return dbHandle;
    }

    GraphLiteFFI ffi() {
        return ffi;
    }

    void ensureOpen() {
        if (closed) {
            throw new ConnectionException(GraphLiteException.ErrorCode.NULL_POINTER.getCode(), "Database is closed");
        }
    }

    @Override
    public void close() {
        if (!closed) {
            closed = true;
            try {
                ffi.graphlite_close(dbHandle);
            } catch (Throwable ignored) {
            } finally {
                arena.close();
            }
        }
    }

    /**
     * Get GraphLite version string.
     */
    public static String version() {
        try (Arena arena = Arena.ofConfined()) {
            GraphLiteFFI ffi = new GraphLiteFFI(arena);
            return ffi.graphlite_version();
        } catch (Throwable t) {
            return "unknown";
        }
    }
}
