package io.graphlite.sdk;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

/**
 * Immutable query result with convenience helpers.
 */
public final class QueryResult implements Iterable<Map<String, Object>> {
    private final List<String> variables;
    private final List<Map<String, Object>> rows;

    QueryResult(String jsonPayload) {
        try {
            JSONObject root = new JSONObject(jsonPayload);
            this.variables = parseVariables(root);
            this.rows = parseRows(root);
        } catch (JSONException e) {
            throw new Errors.SerializationException("Failed to parse GraphLite query JSON payload", e);
        }
    }

    public List<String> variables() {
        return variables;
    }

    public List<String> getVariables() {
        return variables();
    }

    public List<Map<String, Object>> rows() {
        return rows;
    }

    public List<Map<String, Object>> getRows() {
        return rows();
    }

    public int rowCount() {
        return rows.size();
    }

    public int getRowCount() {
        return rowCount();
    }

    public boolean isEmpty() {
        return rows.isEmpty();
    }

    public Map<String, Object> first() {
        return rows.isEmpty() ? null : rows.get(0);
    }

    public List<Object> column(String columnName) {
        List<Object> values = new ArrayList<>(rows.size());
        for (Map<String, Object> row : rows) {
            values.add(row.get(columnName));
        }
        return List.copyOf(values);
    }

    @Override
    public Iterator<Map<String, Object>> iterator() {
        return rows.iterator();
    }

    @Override
    public String toString() {
        return "QueryResult(rows=" + rowCount() + ", variables=" + variables + ")";
    }

    private static List<String> parseVariables(JSONObject root) {
        JSONArray vars = root.optJSONArray("variables");
        if (vars == null || vars.length() == 0) {
            return List.of();
        }

        List<String> parsed = new ArrayList<>(vars.length());
        for (int i = 0; i < vars.length(); i++) {
            parsed.add(vars.optString(i));
        }
        return Collections.unmodifiableList(parsed);
    }

    private static List<Map<String, Object>> parseRows(JSONObject root) {
        JSONArray rawRows = root.optJSONArray("rows");
        if (rawRows == null || rawRows.length() == 0) {
            return List.of();
        }

        List<Map<String, Object>> parsedRows = new ArrayList<>(rawRows.length());
        for (int i = 0; i < rawRows.length(); i++) {
            Object rowObj = rawRows.get(i);
            if (!(rowObj instanceof JSONObject rowJson)) {
                continue;
            }

            JSONObject values = rowJson.optJSONObject("values");
            JSONObject source = values != null ? values : rowJson;
            parsedRows.add(Collections.unmodifiableMap(flattenObject(source)));
        }
        return Collections.unmodifiableList(parsedRows);
    }

    private static Map<String, Object> flattenObject(JSONObject object) {
        Map<String, Object> flattened = new LinkedHashMap<>();
        for (String key : object.keySet()) {
            flattened.put(key, unwrapValue(object.get(key)));
        }
        return flattened;
    }

    private static Object unwrapValue(Object raw) {
        if (raw == null || raw == JSONObject.NULL) {
            return null;
        }
        if (raw instanceof JSONObject jsonObject) {
            if (jsonObject.length() == 1) {
                String tag = jsonObject.keys().next();
                Object inner = jsonObject.get(tag);
                return switch (tag) {
                    case "String", "Boolean" -> unwrapValue(inner);
                    case "Number" -> normalizeNumber(inner);
                    case "Null" -> null;
                    case "List" -> toList(inner);
                    case "Map" -> toMap(inner);
                    default -> toMap(jsonObject);
                };
            }
            return toMap(jsonObject);
        }
        if (raw instanceof JSONArray jsonArray) {
            return toList(jsonArray);
        }
        if (raw instanceof Number) {
            return normalizeNumber(raw);
        }
        return raw;
    }

    private static Object normalizeNumber(Object value) {
        if (!(value instanceof Number number)) {
            return value;
        }

        double asDouble = number.doubleValue();
        if (Double.isFinite(asDouble) && Math.rint(asDouble) == asDouble) {
            if (asDouble >= Integer.MIN_VALUE && asDouble <= Integer.MAX_VALUE) {
                return (int) asDouble;
            }
            if (asDouble >= Long.MIN_VALUE && asDouble <= Long.MAX_VALUE) {
                return (long) asDouble;
            }
        }
        return number;
    }

    private static Map<String, Object> toMap(Object value) {
        if (!(value instanceof JSONObject object)) {
            Map<String, Object> single = new LinkedHashMap<>();
            single.put("value", unwrapValue(value));
            return Collections.unmodifiableMap(single);
        }

        Map<String, Object> map = new LinkedHashMap<>();
        for (String key : object.keySet()) {
            map.put(key, unwrapValue(object.get(key)));
        }
        return Collections.unmodifiableMap(map);
    }

    private static List<Object> toList(Object value) {
        if (!(value instanceof JSONArray array)) {
            List<Object> single = new ArrayList<>(1);
            single.add(unwrapValue(value));
            return Collections.unmodifiableList(single);
        }

        List<Object> list = new ArrayList<>(array.length());
        for (int i = 0; i < array.length(); i++) {
            list.add(unwrapValue(array.get(i)));
        }
        return Collections.unmodifiableList(list);
    }
}
