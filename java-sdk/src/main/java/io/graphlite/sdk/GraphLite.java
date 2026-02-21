package io.graphlite.sdk;

import io.graphlite.sdk.ffi.GraphLiteFFI;
import java.lang.foreign.MemorySegment;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * High-level GraphLite entry point.
 */
public final class GraphLite implements AutoCloseable {
    private final GraphLiteFFI ffi;
    private final Set<String> openSessions;
    private final AtomicBoolean closed;
    private MemorySegment dbHandle;

    private GraphLite(GraphLiteFFI ffi, MemorySegment dbHandle) {
        this.ffi = ffi;
        this.dbHandle = dbHandle;
        this.openSessions = ConcurrentHashMap.newKeySet();
        this.closed = new AtomicBoolean(false);
    }

    /**
     * Open or create a GraphLite database at {@code path}.
     */
    public static GraphLite open(String path) {
        Objects.requireNonNull(path, "path must not be null");
        GraphLiteFFI ffi = GraphLiteFFI.shared();
        MemorySegment dbHandle = ffi.openDatabase(path);
        return new GraphLite(ffi, dbHandle);
    }

    /**
     * Return GraphLite core version as reported by the native library.
     */
    public static String version() {
        return GraphLiteFFI.shared().version();
    }

    /**
     * Create a session for the given user.
     */
    public Session session(String username) {
        Objects.requireNonNull(username, "username must not be null");
        ensureOpen();
        String sessionId = ffi.createSession(dbHandle, username);
        openSessions.add(sessionId);
        return new Session(this, sessionId, username);
    }

    QueryResult runQuery(String sessionId, String query) {
        Objects.requireNonNull(sessionId, "sessionId must not be null");
        Objects.requireNonNull(query, "query must not be null");
        ensureOpen();
        String json = ffi.query(dbHandle, sessionId, query);
        return new QueryResult(json);
    }

    void runExecute(String sessionId, String statement) {
        runQuery(sessionId, statement);
    }

    void closeSession(String sessionId) {
        Objects.requireNonNull(sessionId, "sessionId must not be null");
        if (closed.get()) {
            openSessions.remove(sessionId);
            return;
        }
        ffi.closeSession(dbHandle, sessionId);
        openSessions.remove(sessionId);
    }

    boolean isClosed() {
        return closed.get();
    }

    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) {
            return;
        }

        for (String sessionId : List.copyOf(openSessions)) {
            try {
                ffi.closeSession(dbHandle, sessionId);
            } catch (Errors.SessionException ignored) {
                // Continue cleanup even if individual session close fails.
            }
        }
        openSessions.clear();

        ffi.closeDatabase(dbHandle);
        dbHandle = MemorySegment.NULL;
    }

    private void ensureOpen() {
        if (closed.get()) {
            throw Errors.connection(
                Errors.ErrorCode.NULL_POINTER.code(),
                "Database is closed"
            );
        }
    }
}
