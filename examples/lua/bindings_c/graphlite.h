#ifndef GRAPHLITE_H
#define GRAPHLITE_H

#pragma once

/**
 * Error codes returned by FFI functions
 */
typedef enum GraphLiteErrorCode {
  Success = 0,
  NullPointer = 1,
  InvalidUtf8 = 2,
  DatabaseOpenError = 3,
  SessionError = 4,
  QueryError = 5,
  PanicError = 6,
  JsonError = 7,
} GraphLiteErrorCode;

typedef struct Arc_QueryCoordinator Arc_QueryCoordinator;

typedef struct GraphLiteDB {
  struct Arc_QueryCoordinator coordinator;
} GraphLiteDB;

struct GraphLiteDB *graphlite_open(const char *path, enum GraphLiteErrorCode *error_out);
char *graphlite_create_session(struct GraphLiteDB *db,
                               const char *username,
                               enum GraphLiteErrorCode *error_out);
char *graphlite_query(struct GraphLiteDB *db,
                      const char *session_id,
                      const char *query,
                      enum GraphLiteErrorCode *error_out);
enum GraphLiteErrorCode graphlite_close_session(struct GraphLiteDB *db,
                                                const char *session_id,
                                                enum GraphLiteErrorCode *error_out);
void graphlite_free_string(char *s);
void graphlite_close(struct GraphLiteDB *db);
const char *graphlite_version(void);

#endif  /* GRAPHLITE_H */
