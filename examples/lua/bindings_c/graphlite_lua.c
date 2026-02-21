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
    
    lua_pushstring(L, res);
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