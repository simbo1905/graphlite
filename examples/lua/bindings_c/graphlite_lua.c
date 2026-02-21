/*
 * graphlite_lua.c -- Minimal Lua 5.4 C module for GraphLite.
 *
 * Lua 5.4 has no built-in FFI; this tiny C99 shim calls the GraphLite FFI
 * shared library (the same one used by the Python/Java bindings) and exposes
 * a small Lua-friendly API -- just enough for the BasicUsage demo.
 *
 * JSON decoding is NOT done here.  The query() method returns the raw JSON
 * string from the engine; the Lua caller uses dkjson (installed via
 * luarocks) to decode it.
 *
 * Build (Linux, from examples/lua/bindings_c/):
 *   gcc -std=c99 -shared -fPIC -o graphlite_lua.so graphlite_lua.c \
 *       $(pkg-config --cflags lua5.4) \
 *       -L../../../target/release -lgraphlite_ffi
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include <stdlib.h>
#include <string.h>

#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>

/* ------------------------------------------------------------------ */
/* GraphLite FFI declarations (mirror of graphlite.h)                 */
/* ------------------------------------------------------------------ */

typedef enum {
    GL_SUCCESS           = 0,
    GL_NULL_POINTER      = 1,
    GL_INVALID_UTF8      = 2,
    GL_DATABASE_OPEN_ERR = 3,
    GL_SESSION_ERR       = 4,
    GL_QUERY_ERR         = 5,
    GL_PANIC_ERR         = 6,
    GL_JSON_ERR          = 7
} GraphLiteErrorCode;

typedef struct GraphLiteDB GraphLiteDB;

extern GraphLiteDB        *graphlite_open(const char *path,
                                          GraphLiteErrorCode *err);
extern char               *graphlite_create_session(GraphLiteDB *db,
                                                    const char *username,
                                                    GraphLiteErrorCode *err);
extern char               *graphlite_query(GraphLiteDB *db,
                                           const char *session_id,
                                           const char *query,
                                           GraphLiteErrorCode *err);
extern GraphLiteErrorCode  graphlite_close_session(GraphLiteDB *db,
                                                   const char *session_id,
                                                   GraphLiteErrorCode *err);
extern void                graphlite_free_string(char *s);
extern void                graphlite_close(GraphLiteDB *db);
extern const char         *graphlite_version(void);

/* ------------------------------------------------------------------ */
/* Metatable name for db userdata                                     */
/* ------------------------------------------------------------------ */

#define GRAPHLITE_DB_MT "GraphLiteDB"

typedef struct {
    GraphLiteDB *db;
} LuaGraphLiteDB;

/* ------------------------------------------------------------------ */
/* Helpers                                                            */
/* ------------------------------------------------------------------ */

static const char *errcode_name(GraphLiteErrorCode c) {
    switch (c) {
        case GL_SUCCESS:           return "Success";
        case GL_NULL_POINTER:      return "NullPointer";
        case GL_INVALID_UTF8:      return "InvalidUtf8";
        case GL_DATABASE_OPEN_ERR: return "DatabaseOpenError";
        case GL_SESSION_ERR:       return "SessionError";
        case GL_QUERY_ERR:         return "QueryError";
        case GL_PANIC_ERR:         return "PanicError";
        case GL_JSON_ERR:          return "JsonError";
    }
    return "Unknown";
}

static int raise_gl_error(lua_State *L, GraphLiteErrorCode c,
                          const char *context) {
    return luaL_error(L, "GraphLite error (%s, code %d): %s",
                      errcode_name(c), (int)c, context);
}

static LuaGraphLiteDB *check_db(lua_State *L, int idx) {
    LuaGraphLiteDB *ud =
        (LuaGraphLiteDB *)luaL_checkudata(L, idx, GRAPHLITE_DB_MT);
    if (ud->db == NULL)
        luaL_error(L, "attempt to use a closed GraphLite database");
    return ud;
}

/* ------------------------------------------------------------------ */
/* Module functions                                                   */
/* ------------------------------------------------------------------ */

/* gl.version() -> string */
static int gl_version(lua_State *L) {
    const char *v = graphlite_version();
    lua_pushstring(L, v ? v : "unknown");
    return 1;
}

/* gl.open(path) -> userdata db */
static int gl_open(lua_State *L) {
    const char *path = luaL_checkstring(L, 1);
    GraphLiteErrorCode err = GL_SUCCESS;
    GraphLiteDB *raw = graphlite_open(path, &err);
    if (raw == NULL)
        return raise_gl_error(L, err, "failed to open database");

    LuaGraphLiteDB *ud =
        (LuaGraphLiteDB *)lua_newuserdata(L, sizeof(LuaGraphLiteDB));
    ud->db = raw;
    luaL_setmetatable(L, GRAPHLITE_DB_MT);
    return 1;
}

/* db:create_session(username) -> session_id string */
static int db_create_session(lua_State *L) {
    LuaGraphLiteDB *ud = check_db(L, 1);
    const char *user = luaL_checkstring(L, 2);
    GraphLiteErrorCode err = GL_SUCCESS;
    char *sid = graphlite_create_session(ud->db, user, &err);
    if (sid == NULL)
        return raise_gl_error(L, err, "failed to create session");
    lua_pushstring(L, sid);
    graphlite_free_string(sid);
    return 1;
}

/* db:execute(session_id, query) -> nil  (raises on error) */
static int db_execute(lua_State *L) {
    LuaGraphLiteDB *ud = check_db(L, 1);
    const char *sid   = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode err = GL_SUCCESS;
    char *json = graphlite_query(ud->db, sid, query, &err);
    if (json == NULL && err != GL_SUCCESS)
        return raise_gl_error(L, err, query);
    if (json) graphlite_free_string(json);
    return 0;
}

/* db:query(session_id, query) -> JSON string from engine */
static int db_query(lua_State *L) {
    LuaGraphLiteDB *ud = check_db(L, 1);
    const char *sid   = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode err = GL_SUCCESS;
    char *json = graphlite_query(ud->db, sid, query, &err);
    if (json == NULL) {
        if (err != GL_SUCCESS)
            return raise_gl_error(L, err, query);
        lua_pushliteral(L, "{}");
        return 1;
    }
    lua_pushstring(L, json);
    graphlite_free_string(json);
    return 1;
}

/* db:close_session(session_id) */
static int db_close_session(lua_State *L) {
    LuaGraphLiteDB *ud = check_db(L, 1);
    const char *sid = luaL_checkstring(L, 2);
    GraphLiteErrorCode err = GL_SUCCESS;
    GraphLiteErrorCode rc = graphlite_close_session(ud->db, sid, &err);
    if (rc != GL_SUCCESS)
        return raise_gl_error(L, rc, "failed to close session");
    return 0;
}

/* db:close() */
static int db_close(lua_State *L) {
    LuaGraphLiteDB *ud =
        (LuaGraphLiteDB *)luaL_checkudata(L, 1, GRAPHLITE_DB_MT);
    if (ud->db) {
        graphlite_close(ud->db);
        ud->db = NULL;
    }
    return 0;
}

/* __gc metamethod -- ensure cleanup even if user forgets db:close() */
static int db_gc(lua_State *L) {
    return db_close(L);
}

/* __tostring metamethod */
static int db_tostring(lua_State *L) {
    LuaGraphLiteDB *ud =
        (LuaGraphLiteDB *)luaL_checkudata(L, 1, GRAPHLITE_DB_MT);
    if (ud->db)
        lua_pushfstring(L, "GraphLiteDB (%p)", (void *)ud->db);
    else
        lua_pushliteral(L, "GraphLiteDB (closed)");
    return 1;
}

/* ------------------------------------------------------------------ */
/* Module registration                                                */
/* ------------------------------------------------------------------ */

static const luaL_Reg db_methods[] = {
    { "create_session", db_create_session },
    { "execute",        db_execute        },
    { "query",          db_query          },
    { "close_session",  db_close_session  },
    { "close",          db_close          },
    { NULL, NULL }
};

static const luaL_Reg db_meta[] = {
    { "__gc",       db_gc       },
    { "__tostring", db_tostring },
    { NULL, NULL }
};

static const luaL_Reg module_funcs[] = {
    { "version", gl_version },
    { "open",    gl_open    },
    { NULL, NULL }
};

int luaopen_graphlite_lua(lua_State *L) {
    /* create metatable for GraphLiteDB userdata */
    luaL_newmetatable(L, GRAPHLITE_DB_MT);

    /* mt.__index = mt  (so db:method() works) */
    lua_pushvalue(L, -1);
    lua_setfield(L, -2, "__index");

    luaL_setfuncs(L, db_methods, 0);
    luaL_setfuncs(L, db_meta, 0);
    lua_pop(L, 1);

    /* module table */
    luaL_newlib(L, module_funcs);
    return 1;
}
