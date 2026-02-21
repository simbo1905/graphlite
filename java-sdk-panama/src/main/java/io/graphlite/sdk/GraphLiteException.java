package io.graphlite.sdk;

/**
 * Base exception for all GraphLite SDK errors.
 * Typed exception hierarchy mirrors Python SDK semantics with error codes from the FFI layer.
 */
public sealed class GraphLiteException extends RuntimeException
        permits ConnectionException, SessionException, QueryException, SerializationException {

    private final int errorCode;

    public GraphLiteException(int errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
    }

    public GraphLiteException(int errorCode, String message, Throwable cause) {
        super(message, cause);
        this.errorCode = errorCode;
    }

    public int getErrorCode() {
        return errorCode;
    }

    public ErrorCode getErrorCodeEnum() {
        return ErrorCode.fromInt(errorCode);
    }

    /** FFI error codes matching graphlite.h */
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

        ErrorCode(int code) {
            this.code = code;
        }

        public int getCode() {
            return code;
        }

        public static ErrorCode fromInt(int code) {
            for (ErrorCode ec : values()) {
                if (ec.code == code) {
                    return ec;
                }
            }
            return null;
        }
    }
}
