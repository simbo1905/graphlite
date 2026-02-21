package io.graphlite.sdk;

/**
 * Raised when database connection/open fails.
 */
public final class ConnectionException extends GraphLiteException {
    public ConnectionException(int errorCode, String message) {
        super(errorCode, message);
    }
    public ConnectionException(int errorCode, String message, Throwable cause) {
        super(errorCode, message, cause);
    }
}
