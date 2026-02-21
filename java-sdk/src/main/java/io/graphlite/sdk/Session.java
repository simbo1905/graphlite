package io.graphlite.sdk;

import java.util.Objects;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Session-centric GraphLite API.
 */
public final class Session implements AutoCloseable {
    private final GraphLite db;
    private final String sessionId;
    private final String username;
    private final AtomicBoolean closed;

    Session(GraphLite db, String sessionId, String username) {
        this.db = db;
        this.sessionId = sessionId;
        this.username = username;
        this.closed = new AtomicBoolean(false);
    }

    public String id() {
        return sessionId;
    }

    public String username() {
        return username;
    }

    /**
     * Execute a query and return rows.
     */
    public QueryResult query(String query) {
        Objects.requireNonNull(query, "query must not be null");
        ensureOpen();
        return db.runQuery(sessionId, query);
    }

    /**
     * Execute a statement when no return rows are needed.
     */
    public void execute(String statement) {
        Objects.requireNonNull(statement, "statement must not be null");
        ensureOpen();
        db.runExecute(sessionId, statement);
    }

    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) {
            return;
        }
        db.closeSession(sessionId);
    }

    private void ensureOpen() {
        if (closed.get()) {
            throw Errors.session(
                Errors.ErrorCode.NULL_POINTER.code(),
                "Session is already closed"
            );
        }
        if (db.isClosed()) {
            throw Errors.connection(
                Errors.ErrorCode.NULL_POINTER.code(),
                "Database is closed"
            );
        }
    }
}
