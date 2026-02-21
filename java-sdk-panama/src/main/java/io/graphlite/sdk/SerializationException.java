package io.graphlite.sdk;

/**
 * Raised when JSON serialization/deserialization fails.
 */
public final class SerializationException extends GraphLiteException {
    public SerializationException(int errorCode, String message) {
        super(errorCode, message);
    }

    public SerializationException(String message) {
        super(-1, message);
    }
}
