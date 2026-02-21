package io.graphlite.sdk;

/**
 * Raised when query execution fails.
 */
public final class QueryException extends GraphLiteException {
    public QueryException(int errorCode, String message) {
        super(errorCode, message);
    }
    public QueryException(int errorCode, String message, Throwable cause) {
        super(errorCode, message, cause);
    }
}
