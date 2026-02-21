package io.graphlite.sdk;

import io.graphlite.sdk.Errors.ConnectionException;
import io.graphlite.sdk.Errors.ErrorCode;
import io.graphlite.sdk.Errors.SessionException;
import io.graphlite.sdk.ffi.GraphLiteFFI;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.util.ArrayList;
import java.util.List;

/**
 * High-level entry point for working with a GraphLite database.
 * <p>
 * Uses Java's Foreign Function &amp; Memory API (Project Panama) to call the
 * native {@code libgraphlite_ffi} library &mdash; no JNI required.
 *
 * <pre>{@code
 * try (var db = GraphLite.open("/tmp/mydb")) {
 *     try (var session = db.session("admin")) {
 *         session.execute("CREATE GRAPH IF NOT EXISTS g");
 *         session.execute("SESSION SET GRAPH g");
 *         session.execute("INSERT (:Person {name: 'Alice'})");
 *         var result = session.query("MATCH (p:Person) RETURN p.name");
 *         System.out.println(result.rows());
 *     }
 * }
 * }</pre>
 */
public final class GraphLite implements AutoCloseable {

    private MemorySegment dbHandle;
    private final List<Session> openSessions = new ArrayList<>();

    private GraphLite(MemorySegment dbHandle) {
        this.dbHandle = dbHandle;
    }

    /**
     * Open a GraphLite database at the given directory path.
     * The directory will be created if it does not exist.
     *
     * @param path filesystem path to the database directory
     * @return a new {@link GraphLite} instance (must be closed)
     * @throws ConnectionException if the database cannot be opened
     */
    public static GraphLite open(String path) {
        GraphLiteFFI.init();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cPath = GraphLiteFFI.toCString(path, arena);
            MemorySegment errorOut = GraphLiteFFI.allocErrorOut(arena);

            MemorySegment handle = GraphLiteFFI.open(cPath, errorOut);
            if (handle.equals(MemorySegment.NULL)) {
                ErrorCode ec = GraphLiteFFI.readErrorCode(errorOut);
                throw new ConnectionException(ec, "Failed to open database at " + path);
            }
            return new GraphLite(handle);
        }
    }

    /**
     * Create a new session for the given username.
     *
     * @param username the user identity for this session
     * @return a new {@link Session} (must be closed)
     * @throws SessionException if session creation fails
     */
    public Session session(String username) {
        checkOpen();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cUser = GraphLiteFFI.toCString(username, arena);
            MemorySegment errorOut = GraphLiteFFI.allocErrorOut(arena);

            MemorySegment sessionPtr = GraphLiteFFI.createSession(dbHandle, cUser, errorOut);
            if (sessionPtr.equals(MemorySegment.NULL)) {
                ErrorCode ec = GraphLiteFFI.readErrorCode(errorOut);
                throw new SessionException(ec, "Failed to create session for user '" + username + "'");
            }

            try {
                String sessionId = GraphLiteFFI.fromCString(sessionPtr);
                Session s = new Session(dbHandle, sessionId, username);
                openSessions.add(s);
                return s;
            } finally {
                GraphLiteFFI.freeString(sessionPtr);
            }
        }
    }

    /**
     * Get the native library version string.
     */
    public static String version() {
        GraphLiteFFI.init();
        MemorySegment versionPtr = GraphLiteFFI.version();
        if (versionPtr.equals(MemorySegment.NULL)) return "unknown";
        return GraphLiteFFI.fromCString(versionPtr);
    }

    /**
     * Close the database and all sessions opened through it.
     * Idempotent &mdash; safe to call multiple times.
     */
    @Override
    public void close() {
        if (dbHandle == null) return;
        for (Session s : new ArrayList<>(openSessions)) {
            try { s.close(); } catch (Exception ignored) {}
        }
        openSessions.clear();
        GraphLiteFFI.close(dbHandle);
        dbHandle = null;
    }

    private void checkOpen() {
        if (dbHandle == null) {
            throw new ConnectionException("Database is closed");
        }
    }
}
