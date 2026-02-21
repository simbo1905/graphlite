#include <string.h>

#include <lua.h>
#include <lauxlib.h>

#include "graphlite.h"

#define GRAPHLITE_DB_MT "graphlite_lua.db"

typedef struct {
    GraphLiteDB *db;
} lua_graphlite_db;

static const char *graphlite_error_name(GraphLiteErrorCode code) {
    switch (code) {
        case Success:
            return "Success";
        case NullPointer:
            return "NullPointer";
        case InvalidUtf8:
            return "InvalidUtf8";
        case DatabaseOpenError:
            return "DatabaseOpenError";
        case SessionError:
            return "SessionError";
        case QueryError:
            return "QueryError";
        case PanicError:
            return "PanicError";
        case JsonError:
            return "JsonError";
        default:
            return "UnknownError";
    }
}

static int raise_graphlite_error(
    lua_State *L,
    const char *operation,
    GraphLiteErrorCode code,
    const char *detail
) {
    if (detail == NULL) {
        detail = "operation failed";
    }

    return luaL_error(
        L,
        "GraphLite %s failed (code=%d %s): %s",
        operation,
        (int)code,
        graphlite_error_name(code),
        detail
    );
}

static lua_graphlite_db *check_db(lua_State *L, int index) {
    lua_graphlite_db *db = (lua_graphlite_db *)luaL_checkudata(L, index, GRAPHLITE_DB_MT);
    luaL_argcheck(L, db->db != NULL, index, "database is closed");
    return db;
}

static void close_db_handle(lua_graphlite_db *db) {
    if (db->db != NULL) {
        graphlite_close(db->db);
        db->db = NULL;
    }
}

static int l_graphlite_version(lua_State *L) {
    const char *version = graphlite_version();
    if (version == NULL) {
        lua_pushliteral(L, "unknown");
    } else {
        lua_pushstring(L, version);
    }
    return 1;
}

static int l_graphlite_open(lua_State *L) {
    const char *path = luaL_checkstring(L, 1);
    GraphLiteErrorCode error_code = Success;
    GraphLiteDB *handle = graphlite_open(path, &error_code);
    lua_graphlite_db *db;

    if (handle == NULL) {
        return raise_graphlite_error(L, "open", error_code, path);
    }

    db = (lua_graphlite_db *)lua_newuserdatauv(L, sizeof(*db), 0);
    db->db = handle;

    luaL_getmetatable(L, GRAPHLITE_DB_MT);
    lua_setmetatable(L, -2);

    return 1;
}

static int l_db_create_session(lua_State *L) {
    lua_graphlite_db *db = check_db(L, 1);
    const char *username = luaL_checkstring(L, 2);
    GraphLiteErrorCode error_code = Success;
    char *session_id = graphlite_create_session(db->db, username, &error_code);

    if (session_id == NULL) {
        return raise_graphlite_error(L, "create_session", error_code, username);
    }

    lua_pushstring(L, session_id);
    graphlite_free_string(session_id);
    return 1;
}

static int l_db_execute(lua_State *L) {
    lua_graphlite_db *db = check_db(L, 1);
    const char *session_id = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode error_code = Success;
    char *query_result = graphlite_query(db->db, session_id, query, &error_code);

    if (query_result == NULL) {
        return raise_graphlite_error(L, "execute", error_code, query);
    }

    graphlite_free_string(query_result);
    return 0;
}

static int l_db_query(lua_State *L) {
    lua_graphlite_db *db = check_db(L, 1);
    const char *session_id = luaL_checkstring(L, 2);
    const char *query = luaL_checkstring(L, 3);
    GraphLiteErrorCode error_code = Success;
    char *query_result_json = graphlite_query(db->db, session_id, query, &error_code);

    if (query_result_json == NULL) {
        return raise_graphlite_error(L, "query", error_code, query);
    }

    lua_pushlstring(L, query_result_json, strlen(query_result_json));
    graphlite_free_string(query_result_json);
    return 1;
}

static int l_db_close_session(lua_State *L) {
    lua_graphlite_db *db = check_db(L, 1);
    const char *session_id = luaL_checkstring(L, 2);
    GraphLiteErrorCode error_code = Success;
    GraphLiteErrorCode result = graphlite_close_session(db->db, session_id, &error_code);
    GraphLiteErrorCode effective_error = (result != Success) ? result : error_code;

    if (effective_error != Success) {
        return raise_graphlite_error(L, "close_session", effective_error, session_id);
    }

    return 0;
}

static int l_db_close(lua_State *L) {
    lua_graphlite_db *db = (lua_graphlite_db *)luaL_checkudata(L, 1, GRAPHLITE_DB_MT);
    close_db_handle(db);
    return 0;
}

static int l_db_gc(lua_State *L) {
    lua_graphlite_db *db = (lua_graphlite_db *)luaL_checkudata(L, 1, GRAPHLITE_DB_MT);
    close_db_handle(db);
    return 0;
}

static const luaL_Reg graphlite_db_methods[] = {
    {"create_session", l_db_create_session},
    {"execute", l_db_execute},
    {"query", l_db_query},
    {"close_session", l_db_close_session},
    {"close", l_db_close},
    {NULL, NULL}
};

static const luaL_Reg graphlite_module_functions[] = {
    {"version", l_graphlite_version},
    {"open", l_graphlite_open},
    {NULL, NULL}
};

#if defined(_WIN32)
#define LUA_MODULE_EXPORT __declspec(dllexport)
#else
#define LUA_MODULE_EXPORT
#endif

LUA_MODULE_EXPORT int luaopen_graphlite_lua(lua_State *L) {
    luaL_newmetatable(L, GRAPHLITE_DB_MT);

    lua_pushcfunction(L, l_db_gc);
    lua_setfield(L, -2, "__gc");

    lua_newtable(L);
    luaL_setfuncs(L, graphlite_db_methods, 0);
    lua_setfield(L, -2, "__index");

    lua_pop(L, 1);

    lua_newtable(L);
    luaL_setfuncs(L, graphlite_module_functions, 0);
    return 1;
}
