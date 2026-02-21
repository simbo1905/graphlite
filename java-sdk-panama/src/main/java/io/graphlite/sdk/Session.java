package io.graphlite.sdk;

import io.graphlite.sdk.ffi.GraphLiteFFI;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;

/**
 * GraphLite session - provides user context for executing queries.
 * Mirrors Python SDK: db.session(user) -> Session, session.query/execute.
 */
public final class Session implements AutoCloseable {

    private final String sessionId;
    private final GraphLite db;
    private boolean closed;

    Session(String sessionId, GraphLite db) {
        this.sessionId = sessionId;
        this.db = db;
        this.closed = false;
    }

    public String id() {
        return sessionId;
    }

    /**
     * Execute a GQL query and return results.
     *
     * @param query GQL query string
     * @return QueryResult with rows and metadata
     * @throws QueryException if query execution fails
     */
    public QueryResult query(String query) {
        ensureOpen();
        try (Arena callArena = Arena.ofConfined()) {
            MemorySegment errorOut = callArena.allocate(ValueLayout.JAVA_INT);
            MemorySegment resultPtr = db.ffi().graphlite_query(db.dbHandle(), sessionId, query, errorOut);
            if (resultPtr == null || resultPtr.equals(MemorySegment.NULL)) {
                int code = errorOut.get(ValueLayout.JAVA_INT, 0);
                throw new QueryException(code, "Query failed: " + (query.length() > 100 ? query.substring(0, 100) + "..." : query));
            }
            try {
                String json = GraphLiteFFI.readCString(resultPtr);
                return new QueryResult(json);
            } finally {
                db.ffi().graphlite_free_string(resultPtr);
            }
        } catch (QueryException e) {
            throw e;
        } catch (Throwable t) {
            throw new QueryException(GraphLiteException.ErrorCode.PANIC_ERROR.getCode(),
                "Query failed: " + t.getMessage(), t);
        }
    }

    /**
     * Execute a statement without returning results.
     *
     * @param statement GQL statement to execute
     * @throws QueryException if execution fails
     */
    public void execute(String statement) {
        query(statement);
    }

    void ensureOpen() {
        if (closed) {
            throw new SessionException(GraphLiteException.ErrorCode.NULL_POINTER.getCode(), "Session is closed");
        }
        db.ensureOpen();
    }

    @Override
    public void close() {
        if (!closed) {
            closed = true;
            try {
                try (Arena callArena = Arena.ofConfined()) {
                    MemorySegment errorOut = callArena.allocate(ValueLayout.JAVA_INT);
                    db.ffi().graphlite_close_session(db.dbHandle(), sessionId, errorOut);
                }
            } catch (Throwable ignored) {
            }
        }
    }
}
