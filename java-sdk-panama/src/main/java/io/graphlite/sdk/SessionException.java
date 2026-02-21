package io.graphlite.sdk;

/**
 * Raised when session creation or close fails.
 */
public final class SessionException extends GraphLiteException {
    public SessionException(int errorCode, String message) {
        super(errorCode, message);
    }
    public SessionException(int errorCode, String message, Throwable cause) {
        super(errorCode, message, cause);
    }
}
