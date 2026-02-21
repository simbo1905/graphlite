package io.graphlite.sdk;

/**
 * Exception hierarchy for the GraphLite Java SDK.
 * <p>
 * All exceptions extend {@link GraphLiteException}, which carries an
 * {@link ErrorCode} originating from the native FFI layer.
 */
public final class Errors {

    private Errors() {}

    /** Error codes returned by the native graphlite-ffi library. */
    public enum ErrorCode {
        SUCCESS(0),
        NULL_POINTER(1),
        INVALID_UTF8(2),
        DATABASE_OPEN_ERROR(3),
        SESSION_ERROR(4),
        QUERY_ERROR(5),
        PANIC_ERROR(6),
        JSON_ERROR(7);

        private final int code;

        ErrorCode(int code) { this.code = code; }

        public int code() { return code; }

        public static ErrorCode fromNative(int code) {
            for (ErrorCode ec : values()) {
                if (ec.code == code) return ec;
            }
            return PANIC_ERROR;
        }
    }

    /** Base exception for all GraphLite SDK errors. */
    public static class GraphLiteException extends RuntimeException {
        private final ErrorCode errorCode;

        public GraphLiteException(ErrorCode errorCode, String message) {
            super(message);
            this.errorCode = errorCode;
        }

        public GraphLiteException(ErrorCode errorCode, String message, Throwable cause) {
            super(message, cause);
            this.errorCode = errorCode;
        }

        public ErrorCode errorCode() { return errorCode; }
    }

    /** Thrown when the database cannot be opened. */
    public static class ConnectionException extends GraphLiteException {
        public ConnectionException(String message) {
            super(ErrorCode.DATABASE_OPEN_ERROR, message);
        }

        public ConnectionException(ErrorCode code, String message) {
            super(code, message);
        }
    }

    /** Thrown when session creation or closure fails. */
    public static class SessionException extends GraphLiteException {
        public SessionException(String message) {
            super(ErrorCode.SESSION_ERROR, message);
        }

        public SessionException(ErrorCode code, String message) {
            super(code, message);
        }
    }

    /** Thrown when query execution fails. */
    public static class QueryException extends GraphLiteException {
        public QueryException(String message) {
            super(ErrorCode.QUERY_ERROR, message);
        }

        public QueryException(ErrorCode code, String message) {
            super(code, message);
        }
    }

    /** Thrown when the native library cannot be loaded. */
    public static class LibraryLoadException extends GraphLiteException {
        public LibraryLoadException(String message, Throwable cause) {
            super(ErrorCode.PANIC_ERROR, message, cause);
        }
    }

    /** Thrown when JSON result parsing fails. */
    public static class SerializationException extends GraphLiteException {
        public SerializationException(String message, Throwable cause) {
            super(ErrorCode.JSON_ERROR, message, cause);
        }
    }
}
