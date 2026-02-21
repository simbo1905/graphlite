package io.graphlite.sdk;

/**
 * Typed exception hierarchy and error-code mapping for GraphLite.
 */
public final class Errors {
    private Errors() {
    }

    public enum ErrorCode {
        SUCCESS(0),
        NULL_POINTER(1),
        INVALID_UTF8(2),
        DATABASE_OPEN_ERROR(3),
        SESSION_ERROR(4),
        QUERY_ERROR(5),
        PANIC_ERROR(6),
        JSON_ERROR(7),
        UNKNOWN(-1);

        private final int code;

        ErrorCode(int code) {
            this.code = code;
        }

        public int code() {
            return code;
        }

        public static ErrorCode fromCode(int code) {
            for (ErrorCode value : values()) {
                if (value.code == code) {
                    return value;
                }
            }
            return UNKNOWN;
        }
    }

    public abstract static class GraphLiteException extends RuntimeException {
        private final ErrorCode errorCode;
        private final int rawErrorCode;

        protected GraphLiteException(ErrorCode errorCode, int rawErrorCode, String message) {
            super(message);
            this.errorCode = errorCode;
            this.rawErrorCode = rawErrorCode;
        }

        protected GraphLiteException(ErrorCode errorCode, int rawErrorCode, String message, Throwable cause) {
            super(message, cause);
            this.errorCode = errorCode;
            this.rawErrorCode = rawErrorCode;
        }

        public ErrorCode errorCode() {
            return errorCode;
        }

        public int rawErrorCode() {
            return rawErrorCode;
        }
    }

    public static final class ConnectionException extends GraphLiteException {
        public ConnectionException(ErrorCode errorCode, int rawErrorCode, String message) {
            super(errorCode, rawErrorCode, message);
        }

        public ConnectionException(ErrorCode errorCode, int rawErrorCode, String message, Throwable cause) {
            super(errorCode, rawErrorCode, message, cause);
        }
    }

    public static final class SessionException extends GraphLiteException {
        public SessionException(ErrorCode errorCode, int rawErrorCode, String message) {
            super(errorCode, rawErrorCode, message);
        }

        public SessionException(ErrorCode errorCode, int rawErrorCode, String message, Throwable cause) {
            super(errorCode, rawErrorCode, message, cause);
        }
    }

    public static final class QueryException extends GraphLiteException {
        public QueryException(ErrorCode errorCode, int rawErrorCode, String message) {
            super(errorCode, rawErrorCode, message);
        }

        public QueryException(ErrorCode errorCode, int rawErrorCode, String message, Throwable cause) {
            super(errorCode, rawErrorCode, message, cause);
        }
    }

    public static final class SerializationException extends GraphLiteException {
        public SerializationException(String message) {
            super(ErrorCode.JSON_ERROR, ErrorCode.JSON_ERROR.code(), message);
        }

        public SerializationException(String message, Throwable cause) {
            super(ErrorCode.JSON_ERROR, ErrorCode.JSON_ERROR.code(), message, cause);
        }
    }

    public static final class NativeLibraryException extends GraphLiteException {
        public NativeLibraryException(String message) {
            super(ErrorCode.DATABASE_OPEN_ERROR, ErrorCode.DATABASE_OPEN_ERROR.code(), message);
        }

        public NativeLibraryException(String message, Throwable cause) {
            super(ErrorCode.DATABASE_OPEN_ERROR, ErrorCode.DATABASE_OPEN_ERROR.code(), message, cause);
        }
    }

    public static ConnectionException connection(int rawErrorCode, String message) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new ConnectionException(code, rawErrorCode, message + " (error=" + code + ")");
    }

    public static ConnectionException connection(int rawErrorCode, String message, Throwable cause) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new ConnectionException(code, rawErrorCode, message + " (error=" + code + ")", cause);
    }

    public static SessionException session(int rawErrorCode, String message) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new SessionException(code, rawErrorCode, message + " (error=" + code + ")");
    }

    public static SessionException session(int rawErrorCode, String message, Throwable cause) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new SessionException(code, rawErrorCode, message + " (error=" + code + ")", cause);
    }

    public static QueryException query(int rawErrorCode, String message) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new QueryException(code, rawErrorCode, message + " (error=" + code + ")");
    }

    public static QueryException query(int rawErrorCode, String message, Throwable cause) {
        ErrorCode code = ErrorCode.fromCode(rawErrorCode);
        return new QueryException(code, rawErrorCode, message + " (error=" + code + ")", cause);
    }
}
