/*
 * GraphLite Lua 5.4 C Module
 *
 * Minimal C99 shim that wraps the GraphLite FFI. Lua 5.4 has no built-in FFI;
 * this demo uses a tiny C module to prove Lua can embed/use GraphLite.
 *
 * FFI functions used:
 *   graphlite_open, graphlite_close, graphlite_create_session, graphlite_close_session
 *   graphlite_query, graphlite_free_string, graphlite_version
 *
 * Build: link against libgraphlite_ffi (from cargo build -p graphlite-ffi)
 */

#define LUA_LIB
#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "graphlite.h"

static const char *gl_errstr(GraphLiteErrorCode c) {
    switch (c) {
        case Success: return "Success";
        case NullPointer: return "NullPointer";
        case InvalidUtf8: return "InvalidUtf8";
        case DatabaseOpenError: return "DatabaseOpenError";
        case SessionError: return "SessionError";
        case QueryError: return "QueryError";
        case PanicError: return "PanicError";
        case JsonError: return "JsonError";
        default: return "Unknown";
    }
}

/* Minimal JSON parser for GraphLite result format:
 * {"variables":["a","b"],"rows":[{"values":{"a":{"String":"x"},"b":{"Number":30}}},...]}
 */
typedef struct { const char *p; const char *end; } JState;

static void j_skip(JState *j) {
    while (j->p < j->end && (*j->p == ' ' || *j->p == '\t' || *j->p == '\n' || *j->p == '\r')) j->p++;
}

static int j_match(JState *j, const char *s) {
    size_t n = strlen(s);
    if (j->p + n > j->end || memcmp(j->p, s, n) != 0) return 0;
    j->p += n;
    return 1;
}

static int j_parse_value_wrapper(lua_State *L, JState *j);

/* Parse {"String":"x"}, {"Number":n}, {"Boolean":true}, {"Null":null} */
static int j_parse_value_wrapper(lua_State *L, JState *j) {
    j_skip(j);
    if (j->p >= j->end || *j->p != '{') return 0;
    j->p++;
    j_skip(j);
    if (j->p + 8 <= j->end && memcmp(j->p, "\"String\"", 8) == 0) {
        j->p += 8;
        j_skip(j);
        if (*j->p != ':') return 0;
        j->p++;
        j_skip(j);
        if (*j->p != '"') return 0;
        j->p++;
        const char *start = j->p;
        while (j->p < j->end && *j->p != '"') j->p++;
        lua_pushlstring(L, start, j->p - start);
        if (j->p < j->end) j->p++;
        return 1;
    }
    if (j->p + 8 <= j->end && memcmp(j->p, "\"Number\"", 8) == 0) {
        j->p += 8;
        j_skip(j);
        if (*j->p != ':') return 0;
        j->p++;
        j_skip(j);
        double d = strtod(j->p, (char **)&j->p);
        lua_pushnumber(L, d);
        return 1;
    }
    if (j->p + 10 <= j->end && memcmp(j->p, "\"Boolean\"", 10) == 0) {
        j->p += 10;
        j_skip(j);
        if (*j->p != ':') return 0;
        j->p++;
        j_skip(j);
        if (j->p + 4 <= j->end && memcmp(j->p, "true", 4) == 0) {
            lua_pushboolean(L, 1);
            j->p += 4;
        } else if (j->p + 5 <= j->end && memcmp(j->p, "false", 5) == 0) {
            lua_pushboolean(L, 0);
            j->p += 5;
        } else return 0;
        return 1;
    }
    if (j->p + 7 <= j->end && memcmp(j->p, "\"Null\"", 7) == 0) {
        j->p += 7;
        j_skip(j);
        if (*j->p != ':') return 0;
        j->p++;
        j_skip(j);
        if (j->p + 4 <= j->end && memcmp(j->p, "null", 4) == 0) {
            lua_pushnil(L);
            j->p += 4;
        } else return 0;
        return 1;
    }
    /* Fallback: treat as generic value (e.g. plain number/string) */
    if (*j->p == '"') {
        j->p++;
        const char *start = j->p;
        while (j->p < j->end && *j->p != '"') j->p++;
        lua_pushlstring(L, start, j->p - start);
        if (j->p < j->end) j->p++;
        return 1;
    }
    if ((*j->p >= '0' && *j->p <= '9') || *j->p == '-' || *j->p == '.') {
        double d = strtod(j->p, (char **)&j->p);
        lua_pushnumber(L, d);
        return 1;
    }
    return 0;
}

/* Parse "key": value and push to table at -1 */
static int j_parse_pair(lua_State *L, JState *j) {
    j_skip(j);
    if (j->p >= j->end || *j->p != '"') return 0;
    j->p++;
    const char *kstart = j->p;
    while (j->p < j->end && *j->p != '"') j->p++;
    size_t klen = j->p - kstart;
    if (j->p < j->end) j->p++;
    j_skip(j);
    if (j->p >= j->end || *j->p != ':') return 0;
    j->p++;
    j_skip(j);
    if (j->p < j->end && *j->p == '{') {
        if (!j_parse_value_wrapper(L, j)) return 0;
    } else if (j->p < j->end && *j->p == '"') {
        j->p++;
        const char *vstart = j->p;
        while (j->p < j->end && *j->p != '"') j->p++;
        lua_pushlstring(L, vstart, j->p - vstart);
        if (j->p < j->end) j->p++;
    } else if (j->p < j->end && ((*j->p >= '0' && *j->p <= '9') || *j->p == '-' || *j->p == '.')) {
        double d = strtod(j->p, (char **)&j->p);
        lua_pushnumber(L, d);
    } else return 0;
    lua_pushlstring(L, kstart, klen);
    lua_insert(L, -2);
    lua_settable(L, -3);
    return 1;
}

/* Parse {"values":{...}} row object */
/* Suppress unused warning: may be used via FFI/reflection or kept for completeness */
__attribute__((unused)) static int j_parse_row(lua_State *L, JState *j) {
    j_skip(j);
    if (j->p >= j->end || *j->p != '{') return 0;
    j->p++;
    lua_newtable(L);
    for (;;) {
        j_skip(j);
        if (j->p >= j->end) return 0;
        if (*j->p == '}') { j->p++; return 1; }
        if (!j_parse_pair(L, j)) return 0;
        j_skip(j);
        if (j->p < j->end && *j->p == ',') j->p++;
    }
}

/* Parse "values" object {"name":{...},"age":{...}} - sets fields on table at -1 */
static void j_parse_values_obj(lua_State *L, JState *j) {
    j_skip(j);
    if (j->p >= j->end || *j->p != '{') return;
    j->p++;
    for (;;) {
        j_skip(j);
        if (j->p >= j->end || *j->p == '}') break;
        if (*j->p != '"') break;
        j->p++;
        const char *kstart = j->p;
        while (j->p < j->end && *j->p != '"') j->p++;
        size_t klen = j->p - kstart;
        if (j->p < j->end) j->p++;
        j_skip(j);
        if (j->p >= j->end || *j->p != ':') break;
        j->p++;
        j_skip(j);
    if (j_parse_value_wrapper(L, j)) {
      lua_pushlstring(L, kstart, klen);
      lua_insert(L, -2);
      lua_settable(L, -3);
    }
        j_skip(j);
        if (j->p < j->end && *j->p == ',') j->p++;
    }
}

/* Parse rows array and extract values from each row */
static int j_parse_rows(lua_State *L, JState *j) {
    j_skip(j);
    if (j->p >= j->end || *j->p != '[') return 0;
    j->p++;
    lua_newtable(L);
    int idx = 0;
    for (;;) {
        j_skip(j);
        if (j->p >= j->end || *j->p == ']') break;
        if (*j->p != '{') break;
        lua_newtable(L);
        /* Parse {"values":{...}} */
        if (!j_match(j, "{\"values\":")) break;
        j_parse_values_obj(L, j);
        
        int depth = 1;
        while (j->p < j->end && depth > 0) {
            if (*j->p == '{') depth++;
            else if (*j->p == '}') depth--;
            j->p++;
        }
        
        lua_rawseti(L, -2, ++idx);
        j_skip(j);
        if (j->p < j->end && *j->p == ',') j->p++;
    }
    if (j->p < j->end && *j->p == ']') j->p++;
    return 1;
}

/* Parse full result and push {rows={...}, row_count=N} */
static int json_to_lua(lua_State *L, const char *json, size_t len) {
    JState j = { json, json + len };
    j_skip(&j);
    if (j.p >= j.end || *j.p != '{') return 0;
    j.p++;
    lua_newtable(L);
    lua_newtable(L);
    int row_count = 0;
    for (;;) {
        j_skip(&j);
        if (j.p >= j.end || *j.p == '}') break;
        if (j_match(&j, "\"rows\":")) {
            if (!j_parse_rows(L, &j)) return 0;
            row_count = (int)lua_rawlen(L, -1);
            lua_setfield(L, -2, "rows");
        } else if (j_match(&j, "\"variables\":")) {
            j_skip(&j);
            if (j.p < j.end && *j.p == '[') {
                j.p++;
                lua_newtable(L);
                int vi = 0;
                while (j.p < j.end && *j.p != ']') {
                    j_skip(&j);
                    if (*j.p == '"') {
                        j.p++;
                        const char *s = j.p;
                        while (j.p < j.end && *j.p != '"') j.p++;
                        lua_pushlstring(L, s, j.p - s);
                        lua_rawseti(L, -2, ++vi);
                        if (j.p < j.end) j.p++;
                    }
                    j_skip(&j);
                    if (j.p < j.end && *j.p == ',') j.p++;
                }
                if (j.p < j.end) j.p++;
                lua_setfield(L, -2, "variables");
            }
        } else {
            /* Skip unknown key */
            while (j.p < j.end && *j.p != '"' && *j.p != '}') j.p++;
            if (j.p < j.end && *j.p == '"') {
                j.p++;
                while (j.p < j.end && *j.p != '"') j.p++;
                if (j.p < j.end) j.p++;
                j_skip(&j);
                if (j.p < j.end && *j.p == ':') {
                    j.p++;
                    j_skip(&j);
                    if (*j.p == '{') { int depth = 1; j.p++; while (j.p < j.end && depth) { if (*j.p == '{') depth++; else if (*j.p == '}') depth--; j.p++; } }
                    else if (*j.p == '[') { int depth = 1; j.p++; while (j.p < j.end && depth) { if (*j.p == '[') depth++; else if (*j.p == ']') depth--; j.p++; } }
                    else if (*j.p == '"') { j.p++; while (j.p < j.end && *j.p != '"') j.p++; if (j.p < j.end) j.p++; }
                    else while (j.p < j.end && *j.p != ',' && *j.p != '}' && *j.p != ']') j.p++;
                }
            }
        }
        j_skip(&j);
        if (j.p < j.end && *j.p == ',') j.p++;
    }
    lua_pushinteger(L, row_count);
    lua_setfield(L, -2, "row_count");
    return 1;
}

/* DB userdata */
#define DB_MT "graphlite_db"

static int l_db_gc(lua_State *L) {
    struct GraphLiteDB **ud = (struct GraphLiteDB **)lua_touserdata(L, 1);
    if (ud && *ud) {
        graphlite_close(*ud);
        *ud = NULL;
    }
    return 0;
}

static int l_version(lua_State *L) {
    const char *v = graphlite_version();
    lua_pushstring(L, v ? v : "unknown");
    return 1;
}

static int l_open(lua_State *L) {
    const char *path = luaL_checkstring(L, 1);
    GraphLiteErrorCode err = Success;
    struct GraphLiteDB *db = graphlite_open(path, &err);
    if (!db) {
        return luaL_error(L, "GraphLite error (%s, code %d): failed to open database at %s",
            gl_errstr(err), (int)err, path);
    }
    struct GraphLiteDB **ud = (struct GraphLiteDB **)lua_newuserdatauv(L, sizeof(struct GraphLiteDB *), 0);
    *ud = db;
    luaL_setmetatable(L, DB_MT);
    return 1;
}

static struct GraphLiteDB *check_db(lua_State *L, int idx) {
    struct GraphLiteDB **ud = (struct GraphLiteDB **)luaL_checkudata(L, idx, DB_MT);
    if (!ud || !*ud) luaL_error(L, "database is closed");
    return *ud;
}

static int l_create_session(lua_State *L) {
    struct GraphLiteDB *db = check_db(L, 1);
    const char *user = luaL_checkstring(L, 2);
    GraphLiteErrorCode err = Success;
    char *sid = graphlite_create_session(db, user, &err);
    if (!sid) {
        return luaL_error(L, "GraphLite error (%s, code %d): failed to create session for user '%s'",
            gl_errstr(err), (int)err, user);
    }
    lua_pushstring(L, sid);
    graphlite_free_string(sid);
    return 1;
}

static int l_execute(lua_State *L) {
    struct GraphLiteDB *db = check_db(L, 1);
    const char *sid = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode err = Success;
    char *res = graphlite_query(db, sid, query, &err);
    if (!res) {
        return luaL_error(L, "GraphLite error (%s, code %d): execute failed",
            gl_errstr(err), (int)err);
    }
    graphlite_free_string(res);
    return 0;
}

static int l_query(lua_State *L) {
    struct GraphLiteDB *db = check_db(L, 1);
    const char *sid = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode err = Success;
    char *res = graphlite_query(db, sid, query, &err);
    if (!res) {
        return luaL_error(L, "GraphLite error (%s, code %d): query failed",
            gl_errstr(err), (int)err);
    }
    size_t len = strlen(res);
    if (!json_to_lua(L, res, len)) {
        graphlite_free_string(res);
        return luaL_error(L, "GraphLite error: failed to parse query result JSON");
    }
    graphlite_free_string(res);
    return 1;
}

static int l_close_session(lua_State *L) {
    struct GraphLiteDB *db = check_db(L, 1);
    const char *sid = luaL_checkstring(L, 2);
    GraphLiteErrorCode err = Success;
    GraphLiteErrorCode ret = graphlite_close_session(db, sid, &err);
    if (ret != Success) {
        return luaL_error(L, "GraphLite error (%s, code %d): failed to close session",
            gl_errstr(err), (int)err);
    }
    return 0;
}

static int l_close(lua_State *L) {
    struct GraphLiteDB **ud = (struct GraphLiteDB **)luaL_checkudata(L, 1, DB_MT);
    if (ud && *ud) {
        graphlite_close(*ud);
        *ud = NULL;
    }
    return 0;
}

static const luaL_Reg db_methods[] = {
    {"create_session", l_create_session},
    {"execute", l_execute},
    {"query", l_query},
    {"close_session", l_close_session},
    {"close", l_close},
    {NULL, NULL}
};

/* Suppress unused warning: may be used via FFI/reflection or kept for completeness */
__attribute__((unused)) static int l_db_index(lua_State *L) {
    lua_pushvalue(L, 2);
    lua_gettable(L, lua_upvalueindex(1));
    if (!lua_isnil(L, -1)) return 1;
    lua_pop(L, 1);
    luaL_getmetatable(L, DB_MT);
    lua_pushvalue(L, 2);
    lua_rawget(L, -2);
    return 1;
}

LUALIB_API int luaopen_graphlite_lua(lua_State *L) {
    luaL_newmetatable(L, DB_MT);
    lua_pushcfunction(L, l_db_gc);
    lua_setfield(L, -2, "__gc");
    lua_newtable(L);
    for (const luaL_Reg *r = db_methods; r->name; r++) {
        lua_pushcfunction(L, r->func);
        lua_setfield(L, -2, r->name);
    }
    lua_setfield(L, -2, "__index");
    lua_pop(L, 1);

    lua_newtable(L);
    lua_pushcfunction(L, l_version);
    lua_setfield(L, -2, "version");
    lua_pushcfunction(L, l_open);
    lua_setfield(L, -2, "open");
    return 1;
}
