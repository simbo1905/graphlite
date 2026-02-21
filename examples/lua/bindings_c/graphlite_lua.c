#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <lua.h>
#include <lauxlib.h>

#include "graphlite.h"

#define GRAPHLITE_DB_MT "graphlite_lua.db"

typedef struct {
    GraphLiteDB *db;
} lua_graphlite_db;

typedef struct {
    const char *input;
    const char *cursor;
    char error[256];
} json_parser;

static void json_skip_ws(json_parser *parser);
static int json_parse_value(lua_State *L, json_parser *parser);
static void push_unwrapped_value(lua_State *L, int value_index);

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

static void json_set_error(json_parser *parser, const char *message) {
    if (parser->error[0] != '\0') {
        return;
    }

    snprintf(
        parser->error,
        sizeof(parser->error),
        "JSON parse error at byte %zu: %s",
        (size_t)(parser->cursor - parser->input),
        message
    );
}

static int json_match_literal(json_parser *parser, const char *literal) {
    size_t length = strlen(literal);
    if (strncmp(parser->cursor, literal, length) == 0) {
        parser->cursor += length;
        return 1;
    }
    return 0;
}

static int json_parse_hex4(const char *text, unsigned int *codepoint_out) {
    unsigned int value = 0;
    int i;

    for (i = 0; i < 4; i++) {
        char c = text[i];
        value <<= 4;

        if (c >= '0' && c <= '9') {
            value |= (unsigned int)(c - '0');
        } else if (c >= 'a' && c <= 'f') {
            value |= (unsigned int)(c - 'a' + 10);
        } else if (c >= 'A' && c <= 'F') {
            value |= (unsigned int)(c - 'A' + 10);
        } else {
            return 0;
        }
    }

    *codepoint_out = value;
    return 1;
}

static int json_has_n_chars(const char *text, size_t count) {
    size_t i;
    for (i = 0; i < count; i++) {
        if (text[i] == '\0') {
            return 0;
        }
    }
    return 1;
}

static void json_add_utf8(luaL_Buffer *buffer, unsigned int codepoint) {
    if (codepoint <= 0x7F) {
        luaL_addchar(buffer, (char)codepoint);
    } else if (codepoint <= 0x7FF) {
        luaL_addchar(buffer, (char)(0xC0 | (codepoint >> 6)));
        luaL_addchar(buffer, (char)(0x80 | (codepoint & 0x3F)));
    } else if (codepoint <= 0xFFFF) {
        luaL_addchar(buffer, (char)(0xE0 | (codepoint >> 12)));
        luaL_addchar(buffer, (char)(0x80 | ((codepoint >> 6) & 0x3F)));
        luaL_addchar(buffer, (char)(0x80 | (codepoint & 0x3F)));
    } else if (codepoint <= 0x10FFFF) {
        luaL_addchar(buffer, (char)(0xF0 | (codepoint >> 18)));
        luaL_addchar(buffer, (char)(0x80 | ((codepoint >> 12) & 0x3F)));
        luaL_addchar(buffer, (char)(0x80 | ((codepoint >> 6) & 0x3F)));
        luaL_addchar(buffer, (char)(0x80 | (codepoint & 0x3F)));
    } else {
        luaL_addchar(buffer, '?');
    }
}

static int json_parse_string(lua_State *L, json_parser *parser) {
    luaL_Buffer buffer;

    if (*parser->cursor != '"') {
        json_set_error(parser, "expected string");
        return 0;
    }

    parser->cursor++;
    luaL_buffinit(L, &buffer);

    while (*parser->cursor != '\0') {
        unsigned char c = (unsigned char)(*parser->cursor++);

        if (c == '"') {
            luaL_pushresult(&buffer);
            return 1;
        }

        if (c == '\\') {
            char escape = *parser->cursor++;

            if (escape == '\0') {
                json_set_error(parser, "unterminated escape sequence");
                return 0;
            }

            switch (escape) {
                case '"':
                    luaL_addchar(&buffer, '"');
                    break;
                case '\\':
                    luaL_addchar(&buffer, '\\');
                    break;
                case '/':
                    luaL_addchar(&buffer, '/');
                    break;
                case 'b':
                    luaL_addchar(&buffer, '\b');
                    break;
                case 'f':
                    luaL_addchar(&buffer, '\f');
                    break;
                case 'n':
                    luaL_addchar(&buffer, '\n');
                    break;
                case 'r':
                    luaL_addchar(&buffer, '\r');
                    break;
                case 't':
                    luaL_addchar(&buffer, '\t');
                    break;
                case 'u': {
                    unsigned int codepoint;

                    if (!json_has_n_chars(parser->cursor, 4)) {
                        json_set_error(parser, "truncated unicode escape");
                        return 0;
                    }

                    if (!json_parse_hex4(parser->cursor, &codepoint)) {
                        json_set_error(parser, "invalid unicode escape");
                        return 0;
                    }
                    parser->cursor += 4;

                    if (codepoint >= 0xD800 && codepoint <= 0xDBFF) {
                        unsigned int low_surrogate;

                        if (parser->cursor[0] != '\\' || parser->cursor[1] != 'u') {
                            json_set_error(parser, "missing low surrogate in unicode pair");
                            return 0;
                        }
                        parser->cursor += 2;

                        if (!json_has_n_chars(parser->cursor, 4)) {
                            json_set_error(parser, "truncated low surrogate");
                            return 0;
                        }

                        if (!json_parse_hex4(parser->cursor, &low_surrogate)) {
                            json_set_error(parser, "invalid low surrogate");
                            return 0;
                        }
                        parser->cursor += 4;

                        if (low_surrogate < 0xDC00 || low_surrogate > 0xDFFF) {
                            json_set_error(parser, "invalid low surrogate range");
                            return 0;
                        }

                        codepoint = 0x10000 + (((codepoint - 0xD800) << 10) | (low_surrogate - 0xDC00));
                    } else if (codepoint >= 0xDC00 && codepoint <= 0xDFFF) {
                        json_set_error(parser, "unexpected low surrogate");
                        return 0;
                    }

                    json_add_utf8(&buffer, codepoint);
                    break;
                }
                default:
                    json_set_error(parser, "invalid string escape");
                    return 0;
            }
        } else {
            if (c < 0x20) {
                json_set_error(parser, "control character in string");
                return 0;
            }
            luaL_addchar(&buffer, (char)c);
        }
    }

    json_set_error(parser, "unterminated string");
    return 0;
}

static int json_parse_number(lua_State *L, json_parser *parser) {
    char *endptr;
    double value;

    errno = 0;
    value = strtod(parser->cursor, &endptr);

    if (endptr == parser->cursor) {
        json_set_error(parser, "invalid number");
        return 0;
    }

    if (errno == ERANGE || !isfinite(value)) {
        json_set_error(parser, "number out of range");
        return 0;
    }

    parser->cursor = endptr;
    lua_pushnumber(L, value);
    return 1;
}

static int json_parse_array(lua_State *L, json_parser *parser) {
    lua_Integer index = 1;
    int array_index;

    if (*parser->cursor != '[') {
        json_set_error(parser, "expected '['");
        return 0;
    }

    parser->cursor++;
    json_skip_ws(parser);

    lua_newtable(L);
    array_index = lua_gettop(L);

    if (*parser->cursor == ']') {
        parser->cursor++;
        return 1;
    }

    while (1) {
        if (!json_parse_value(L, parser)) {
            lua_pop(L, 1);
            return 0;
        }

        lua_seti(L, array_index, index++);
        json_skip_ws(parser);

        if (*parser->cursor == ',') {
            parser->cursor++;
            json_skip_ws(parser);
            continue;
        }

        if (*parser->cursor == ']') {
            parser->cursor++;
            return 1;
        }

        json_set_error(parser, "expected ',' or ']'");
        lua_pop(L, 1);
        return 0;
    }
}

static int json_parse_object(lua_State *L, json_parser *parser) {
    int object_index;

    if (*parser->cursor != '{') {
        json_set_error(parser, "expected '{'");
        return 0;
    }

    parser->cursor++;
    json_skip_ws(parser);

    lua_newtable(L);
    object_index = lua_gettop(L);

    if (*parser->cursor == '}') {
        parser->cursor++;
        return 1;
    }

    while (1) {
        if (!json_parse_string(L, parser)) {
            lua_pop(L, 1);
            return 0;
        }

        json_skip_ws(parser);
        if (*parser->cursor != ':') {
            json_set_error(parser, "expected ':' after object key");
            lua_pop(L, 2);
            return 0;
        }

        parser->cursor++;
        json_skip_ws(parser);

        if (!json_parse_value(L, parser)) {
            lua_pop(L, 2);
            return 0;
        }

        lua_settable(L, object_index);
        json_skip_ws(parser);

        if (*parser->cursor == ',') {
            parser->cursor++;
            json_skip_ws(parser);
            continue;
        }

        if (*parser->cursor == '}') {
            parser->cursor++;
            return 1;
        }

        json_set_error(parser, "expected ',' or '}'");
        lua_pop(L, 1);
        return 0;
    }
}

static int json_parse_value(lua_State *L, json_parser *parser) {
    json_skip_ws(parser);

    switch (*parser->cursor) {
        case '{':
            return json_parse_object(L, parser);
        case '[':
            return json_parse_array(L, parser);
        case '"':
            return json_parse_string(L, parser);
        case 't':
            if (!json_match_literal(parser, "true")) {
                json_set_error(parser, "invalid literal");
                return 0;
            }
            lua_pushboolean(L, 1);
            return 1;
        case 'f':
            if (!json_match_literal(parser, "false")) {
                json_set_error(parser, "invalid literal");
                return 0;
            }
            lua_pushboolean(L, 0);
            return 1;
        case 'n':
            if (!json_match_literal(parser, "null")) {
                json_set_error(parser, "invalid literal");
                return 0;
            }
            lua_pushnil(L);
            return 1;
        case '-':
            return json_parse_number(L, parser);
        default:
            if (isdigit((unsigned char)*parser->cursor)) {
                return json_parse_number(L, parser);
            }
            json_set_error(parser, "unexpected token");
            return 0;
    }
}

static void json_skip_ws(json_parser *parser) {
    while (*parser->cursor != '\0' && isspace((unsigned char)*parser->cursor)) {
        parser->cursor++;
    }
}

static int parse_json_document(lua_State *L, const char *json, char *error_out, size_t error_out_size) {
    json_parser parser;

    parser.input = json;
    parser.cursor = json;
    parser.error[0] = '\0';

    if (!json_parse_value(L, &parser)) {
        snprintf(error_out, error_out_size, "%s", parser.error);
        return 0;
    }

    json_skip_ws(&parser);
    if (*parser.cursor != '\0') {
        lua_pop(L, 1);
        json_set_error(&parser, "trailing data after JSON document");
        snprintf(error_out, error_out_size, "%s", parser.error);
        return 0;
    }

    return 1;
}

static int is_graphlite_variant_tag(const char *tag) {
    return strcmp(tag, "String") == 0 ||
           strcmp(tag, "Number") == 0 ||
           strcmp(tag, "Boolean") == 0 ||
           strcmp(tag, "DateTime") == 0 ||
           strcmp(tag, "DateTimeWithFixedOffset") == 0 ||
           strcmp(tag, "DateTimeWithNamedTz") == 0 ||
           strcmp(tag, "TimeWindow") == 0 ||
           strcmp(tag, "Array") == 0 ||
           strcmp(tag, "List") == 0 ||
           strcmp(tag, "Vector") == 0 ||
           strcmp(tag, "Path") == 0 ||
           strcmp(tag, "Node") == 0 ||
           strcmp(tag, "Edge") == 0 ||
           strcmp(tag, "Temporal") == 0 ||
           strcmp(tag, "Map") == 0 ||
           strcmp(tag, "Null") == 0;
}

static const char *single_variant_tag(lua_State *L, int table_index) {
    const char *tag = NULL;
    int key_count = 0;

    table_index = lua_absindex(L, table_index);
    lua_pushnil(L);

    while (lua_next(L, table_index) != 0) {
        key_count++;

        if (key_count == 1 && lua_type(L, -2) == LUA_TSTRING) {
            tag = lua_tostring(L, -2);
        } else if (key_count == 1) {
            tag = NULL;
        }

        lua_pop(L, 1);

        if (key_count > 1) {
            lua_pop(L, 1);
            return NULL;
        }
    }

    if (key_count == 1 && tag != NULL && is_graphlite_variant_tag(tag)) {
        return tag;
    }

    return NULL;
}

static int is_array_index_key(lua_State *L, int key_index, size_t array_length) {
    lua_Integer key_value;

    if (!lua_isinteger(L, key_index)) {
        return 0;
    }

    key_value = lua_tointeger(L, key_index);
    if (key_value < 1) {
        return 0;
    }

    return (size_t)key_value <= array_length;
}

static void push_unwrapped_table(lua_State *L, int table_index) {
    size_t array_length;
    size_t i;
    int output_index;

    table_index = lua_absindex(L, table_index);
    lua_newtable(L);
    output_index = lua_gettop(L);

    array_length = lua_rawlen(L, table_index);
    for (i = 1; i <= array_length; i++) {
        lua_geti(L, table_index, (lua_Integer)i);
        push_unwrapped_value(L, -1);
        lua_seti(L, output_index, (lua_Integer)i);
        lua_pop(L, 1);
    }

    lua_pushnil(L);
    while (lua_next(L, table_index) != 0) {
        if (is_array_index_key(L, -2, array_length)) {
            lua_pop(L, 1);
            continue;
        }

        lua_pushvalue(L, -2);
        push_unwrapped_value(L, -2);
        lua_settable(L, output_index);
        lua_pop(L, 1);
    }
}

static void push_unwrapped_value(lua_State *L, int value_index) {
    const char *tag;

    value_index = lua_absindex(L, value_index);

    if (!lua_istable(L, value_index)) {
        lua_pushvalue(L, value_index);
        return;
    }

    tag = single_variant_tag(L, value_index);
    if (tag != NULL) {
        if (strcmp(tag, "Null") == 0) {
            lua_pushnil(L);
            return;
        }

        lua_getfield(L, value_index, tag);
        push_unwrapped_value(L, -1);
        lua_remove(L, -2);
        return;
    }

    push_unwrapped_table(L, value_index);
}

static void push_flattened_row(lua_State *L, int row_index) {
    int values_index;
    int has_values_table = 0;
    int row_output_index;

    row_index = lua_absindex(L, row_index);
    lua_newtable(L);
    row_output_index = lua_gettop(L);

    values_index = row_index;
    lua_getfield(L, row_index, "values");
    if (lua_istable(L, -1)) {
        values_index = lua_gettop(L);
        has_values_table = 1;
    } else {
        lua_pop(L, 1);
    }

    lua_pushnil(L);
    while (lua_next(L, values_index) != 0) {
        if (lua_type(L, -2) == LUA_TSTRING) {
            lua_pushvalue(L, -2);
            push_unwrapped_value(L, -2);
            lua_settable(L, row_output_index);
        }
        lua_pop(L, 1);
    }

    if (has_values_table) {
        lua_pop(L, 1);
    }
}

static void push_query_result_table(lua_State *L, int root_index) {
    int result_index;
    int rows_output_index;
    int row_count = 0;
    size_t rows_length = 0;
    size_t i;

    root_index = lua_absindex(L, root_index);

    lua_newtable(L);
    result_index = lua_gettop(L);

    lua_newtable(L);
    rows_output_index = lua_gettop(L);

    lua_getfield(L, root_index, "rows");
    if (lua_istable(L, -1)) {
        rows_length = lua_rawlen(L, -1);
        for (i = 1; i <= rows_length; i++) {
            lua_geti(L, -1, (lua_Integer)i);
            if (lua_istable(L, -1)) {
                push_flattened_row(L, -1);
                row_count++;
                lua_seti(L, rows_output_index, row_count);
            }
            lua_pop(L, 1);
        }
    }
    lua_pop(L, 1);

    lua_pushinteger(L, row_count);
    lua_setfield(L, result_index, "row_count");

    lua_setfield(L, result_index, "rows");

    lua_getfield(L, root_index, "variables");
    if (lua_istable(L, -1)) {
        push_unwrapped_value(L, -1);
        lua_setfield(L, result_index, "variables");
        lua_pop(L, 1);
    } else {
        lua_pop(L, 1);
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
    char parse_error[256];
    int root_index;

    if (query_result_json == NULL) {
        return raise_graphlite_error(L, "query", error_code, query);
    }

    if (!parse_json_document(L, query_result_json, parse_error, sizeof(parse_error))) {
        graphlite_free_string(query_result_json);
        return luaL_error(L, "Failed to decode GraphLite JSON result: %s", parse_error);
    }
    graphlite_free_string(query_result_json);

    if (!lua_istable(L, -1)) {
        lua_pop(L, 1);
        return luaL_error(L, "GraphLite query returned non-object JSON");
    }

    root_index = lua_absindex(L, -1);
    push_query_result_table(L, root_index);
    lua_remove(L, root_index);

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
