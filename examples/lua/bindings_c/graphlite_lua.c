/*
 * graphlite_lua.c -- Minimal Lua 5.4 C module for GraphLite.
 *
 * Lua 5.4 has no built-in FFI; this tiny C99 shim calls the GraphLite FFI
 * shared library (the same one used by the Python/Java bindings) and exposes
 * a small Lua-friendly API -- just enough for the BasicUsage demo.
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
/* Minimal JSON-to-Lua-table parser                                   */
/*                                                                    */
/* Only handles the subset produced by graphlite_query:               */
/*   { "variables": [...], "rows": [ {k:v, ...}, ... ], "row_count":N}*/
/* Supports: strings, numbers (int/float), true, false, null, arrays, */
/* objects.  Escaped characters in strings are handled.               */
/* ------------------------------------------------------------------ */

typedef struct {
    const char *s;
    size_t      pos;
    size_t      len;
} JsonCtx;

static void json_skip_ws(JsonCtx *j) {
    while (j->pos < j->len) {
        char c = j->s[j->pos];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r')
            j->pos++;
        else
            break;
    }
}

static int json_peek(JsonCtx *j) {
    json_skip_ws(j);
    return (j->pos < j->len) ? (unsigned char)j->s[j->pos] : -1;
}

static int json_parse_value(lua_State *L, JsonCtx *j);

static int json_parse_string(lua_State *L, JsonCtx *j) {
    if (j->s[j->pos] != '"') return 0;
    j->pos++;
    luaL_Buffer buf;
    luaL_buffinit(L, &buf);
    while (j->pos < j->len) {
        char c = j->s[j->pos++];
        if (c == '"') {
            luaL_pushresult(&buf);
            return 1;
        }
        if (c == '\\' && j->pos < j->len) {
            char e = j->s[j->pos++];
            switch (e) {
                case '"':  luaL_addchar(&buf, '"');  break;
                case '\\': luaL_addchar(&buf, '\\'); break;
                case '/':  luaL_addchar(&buf, '/');  break;
                case 'b':  luaL_addchar(&buf, '\b'); break;
                case 'f':  luaL_addchar(&buf, '\f'); break;
                case 'n':  luaL_addchar(&buf, '\n'); break;
                case 'r':  luaL_addchar(&buf, '\r'); break;
                case 't':  luaL_addchar(&buf, '\t'); break;
                case 'u': {
                    /* Basic \uXXXX -- just pass codepoint < 128 as-is,
                       higher codepoints as '?' for this demo. */
                    unsigned cp = 0;
                    for (int i = 0; i < 4 && j->pos < j->len; i++, j->pos++) {
                        char h = j->s[j->pos];
                        cp <<= 4;
                        if (h >= '0' && h <= '9')      cp |= (unsigned)(h - '0');
                        else if (h >= 'a' && h <= 'f') cp |= (unsigned)(h - 'a' + 10);
                        else if (h >= 'A' && h <= 'F') cp |= (unsigned)(h - 'A' + 10);
                    }
                    if (cp < 128)
                        luaL_addchar(&buf, (char)cp);
                    else
                        luaL_addchar(&buf, '?');
                    break;
                }
                default: luaL_addchar(&buf, e); break;
            }
        } else {
            luaL_addchar(&buf, c);
        }
    }
    luaL_pushresult(&buf);
    return 1;
}

static int json_parse_number(lua_State *L, JsonCtx *j) {
    const char *start = j->s + j->pos;
    char *end = NULL;
    int is_float = 0;

    /* scan ahead to classify */
    size_t k = j->pos;
    if (k < j->len && j->s[k] == '-') k++;
    while (k < j->len && j->s[k] >= '0' && j->s[k] <= '9') k++;
    if (k < j->len && (j->s[k] == '.' || j->s[k] == 'e' || j->s[k] == 'E'))
        is_float = 1;

    if (is_float) {
        double d = strtod(start, &end);
        j->pos = (size_t)(end - j->s);
        lua_pushnumber(L, d);
    } else {
        long long ll = strtoll(start, &end, 10);
        j->pos = (size_t)(end - j->s);
        lua_pushinteger(L, (lua_Integer)ll);
    }
    return 1;
}

static int json_parse_array(lua_State *L, JsonCtx *j) {
    j->pos++; /* skip '[' */
    lua_newtable(L);
    int idx = 1;
    if (json_peek(j) == ']') { j->pos++; return 1; }
    for (;;) {
        json_parse_value(L, j);
        lua_rawseti(L, -2, idx++);
        json_skip_ws(j);
        if (j->pos < j->len && j->s[j->pos] == ',') { j->pos++; continue; }
        if (j->pos < j->len && j->s[j->pos] == ']') { j->pos++; break; }
        break;
    }
    return 1;
}

static int json_parse_object(lua_State *L, JsonCtx *j) {
    j->pos++; /* skip '{' */
    lua_newtable(L);
    if (json_peek(j) == '}') { j->pos++; return 1; }
    for (;;) {
        json_skip_ws(j);
        json_parse_string(L, j); /* key */
        json_skip_ws(j);
        if (j->pos < j->len && j->s[j->pos] == ':') j->pos++;
        json_parse_value(L, j); /* value */
        lua_settable(L, -3);
        json_skip_ws(j);
        if (j->pos < j->len && j->s[j->pos] == ',') { j->pos++; continue; }
        if (j->pos < j->len && j->s[j->pos] == '}') { j->pos++; break; }
        break;
    }
    return 1;
}

static int json_parse_value(lua_State *L, JsonCtx *j) {
    int c = json_peek(j);
    if (c == '"')                                   return json_parse_string(L, j);
    if (c == '{')                                   return json_parse_object(L, j);
    if (c == '[')                                   return json_parse_array(L, j);
    if (c == '-' || (c >= '0' && c <= '9'))         return json_parse_number(L, j);
    if (j->len - j->pos >= 4 && memcmp(j->s + j->pos, "true", 4) == 0) {
        j->pos += 4; lua_pushboolean(L, 1); return 1;
    }
    if (j->len - j->pos >= 5 && memcmp(j->s + j->pos, "false", 5) == 0) {
        j->pos += 5; lua_pushboolean(L, 0); return 1;
    }
    if (j->len - j->pos >= 4 && memcmp(j->s + j->pos, "null", 4) == 0) {
        j->pos += 4; lua_pushnil(L); return 1;
    }
    lua_pushnil(L);
    return 1;
}

static void push_json(lua_State *L, const char *json, size_t len) {
    JsonCtx j = { json, 0, len };
    json_parse_value(L, &j);
}

/* ------------------------------------------------------------------ */
/* Post-process query result into flat rows                           */
/*                                                                    */
/* Actual FFI JSON looks like:                                        */
/*   { "variables": ["name","age"],                                   */
/*     "rows_affected": 3,                                            */
/*     "rows": [ { "values": { "name": {"String":"Alice"},            */
/*                              "age":  {"Number":30} }, ... } ] }    */
/*                                                                    */
/* We transform each row from row.values into a flat {name=v,...}     */
/* table and expose rows_affected as row_count.                       */
/* ------------------------------------------------------------------ */

static void flatten_query_result(lua_State *L, int tbl_idx) {
    tbl_idx = lua_absindex(L, tbl_idx);

    /* row_count = rows_affected */
    lua_getfield(L, tbl_idx, "rows_affected");
    if (!lua_isnil(L, -1))
        lua_setfield(L, tbl_idx, "row_count");
    else
        lua_pop(L, 1);

    lua_getfield(L, tbl_idx, "rows");
    if (!lua_istable(L, -1)) { lua_pop(L, 1); return; }
    int rows_idx = lua_absindex(L, -1);

    lua_Integer n = luaL_len(L, rows_idx);
    for (lua_Integer i = 1; i <= n; i++) {
        lua_rawgeti(L, rows_idx, i);              /* row */
        int row_idx = lua_absindex(L, -1);

        lua_getfield(L, row_idx, "values");       /* row.values */
        if (!lua_istable(L, -1)) { lua_pop(L, 2); continue; }
        int vals_idx = lua_absindex(L, -1);

        lua_newtable(L);                          /* flat row */
        int flat_idx = lua_absindex(L, -1);

        lua_pushnil(L);
        while (lua_next(L, vals_idx) != 0) {
            /* stack: key, wrapped_value */
            if (lua_istable(L, -1)) {
                /* Unwrap {"Type": actual_value} -- take the first pair */
                int wrap_idx = lua_absindex(L, -1);
                lua_pushnil(L);
                if (lua_next(L, wrap_idx) != 0) {
                    /* stack: ..., type_name, actual_value */
                    /* flat[key] = actual_value */
                    lua_pushvalue(L, -4);         /* copy original key */
                    lua_pushvalue(L, -2);         /* copy actual value */
                    lua_settable(L, flat_idx);
                    lua_pop(L, 2);                /* type_name, actual_value */
                }
            } else {
                /* Not wrapped -- use as-is */
                lua_pushvalue(L, -2);             /* key */
                lua_pushvalue(L, -2);             /* value */
                lua_settable(L, flat_idx);
            }
            lua_pop(L, 1);                        /* pop value, keep key */
        }

        /* rows[i] = flat_row */
        lua_rawseti(L, rows_idx, i);

        lua_pop(L, 2);                            /* pop vals, row */
    }
    lua_pop(L, 1);                                /* pop rows */
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

/*
 * db:query(session_id, query) -> table
 *   { variables = { "col1", ... },
 *     rows      = { { col1=val, ... }, ... },
 *     row_count = N }
 */
static int db_query(lua_State *L) {
    LuaGraphLiteDB *ud = check_db(L, 1);
    const char *sid   = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode err = GL_SUCCESS;
    char *json = graphlite_query(ud->db, sid, query, &err);
    if (json == NULL) {
        if (err != GL_SUCCESS)
            return raise_gl_error(L, err, query);
        lua_newtable(L);
        return 1;
    }
    push_json(L, json, strlen(json));
    graphlite_free_string(json);
    if (lua_istable(L, -1))
        flatten_query_result(L, -1);
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
