package io.graphlite.sdk;

import io.graphlite.sdk.Errors.ErrorCode;
import io.graphlite.sdk.Errors.QueryException;
import io.graphlite.sdk.Errors.SessionException;
import io.graphlite.sdk.ffi.GraphLiteFFI;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

/**
 * A user session against a GraphLite database.
 * <p>
 * Sessions are obtained from {@link GraphLite#session(String)} and must be
 * closed when no longer needed.  Implements {@link AutoCloseable} for use
 * with try-with-resources.
 *
 * <pre>{@code
 * try (var db = GraphLite.open("/tmp/mydb");
 *      var session = db.session("analyst")) {
 *     session.execute("CREATE GRAPH IF NOT EXISTS g");
 *     session.execute("SESSION SET GRAPH g");
 *     var result = session.query("MATCH (n) RETURN n");
 * }
 * }</pre>
 */
public final class Session implements AutoCloseable {

    private final MemorySegment dbHandle;
    private final String sessionId;
    private final String username;
    private volatile boolean closed;

    Session(MemorySegment dbHandle, String sessionId, String username) {
        this.dbHandle = dbHandle;
        this.sessionId = sessionId;
        this.username = username;
    }

    /** The server-assigned session identifier. */
    public String id() { return sessionId; }

    /** The username this session was created for. */
    public String username() { return username; }

    /**
     * Execute a GQL query and return results.
     *
     * @param gql the GQL query string
     * @return parsed {@link QueryResult}
     * @throws QueryException if execution fails
     */
    public QueryResult query(String gql) {
        checkOpen();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cSession = GraphLiteFFI.toCString(sessionId, arena);
            MemorySegment cQuery = GraphLiteFFI.toCString(gql, arena);
            MemorySegment errorOut = GraphLiteFFI.allocErrorOut(arena);

            MemorySegment resultPtr = GraphLiteFFI.query(dbHandle, cSession, cQuery, errorOut);
            if (resultPtr.equals(MemorySegment.NULL)) {
                ErrorCode ec = GraphLiteFFI.readErrorCode(errorOut);
                throw new QueryException(ec, "Query failed: " + truncate(gql));
            }

            try {
                String json = GraphLiteFFI.fromCString(resultPtr);
                return new QueryResult(json);
            } finally {
                GraphLiteFFI.freeString(resultPtr);
            }
        }
    }

    /**
     * Execute a GQL statement that does not return meaningful results
     * (e.g. INSERT, CREATE, SESSION SET).
     *
     * @param gql the GQL statement
     * @throws QueryException if execution fails
     */
    public void execute(String gql) {
        query(gql);
    }

    /**
     * Close this session, releasing server-side resources.
     * Idempotent &mdash; safe to call multiple times.
     */
    @Override
    public void close() {
        if (closed) return;
        closed = true;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cSession = GraphLiteFFI.toCString(sessionId, arena);
            MemorySegment errorOut = GraphLiteFFI.allocErrorOut(arena);

            int rc = GraphLiteFFI.closeSession(dbHandle, cSession, errorOut);
            if (rc != 0) {
                ErrorCode ec = GraphLiteFFI.readErrorCode(errorOut);
                throw new SessionException(ec, "Failed to close session " + sessionId);
            }
        }
    }

    private void checkOpen() {
        if (closed) throw new SessionException("Session is closed");
    }

    private static String truncate(String s) {
        return s.length() <= 120 ? s : s.substring(0, 117) + "...";
    }
}
