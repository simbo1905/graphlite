package io.graphlite.sdk;

import org.json.JSONArray;
import org.json.JSONObject;

import java.util.*;

/**
 * Query result with row iteration and convenience helpers.
 * Mirrors Python SDK QueryResult semantics (first(), isEmpty(), column()).
 */
public class QueryResult {
    private final List<String> variables;
    private final List<Map<String, Object>> rows;

    public QueryResult(String jsonString) {
        JSONObject data = new JSONObject(jsonString);
        this.variables = parseVariables(data);
        this.rows = parseRows(data);
    }

    private static List<String> parseVariables(JSONObject data) {
        List<String> vars = new ArrayList<>();
        JSONArray arr = data.optJSONArray("variables");
        if (arr != null) {
            for (int i = 0; i < arr.length(); i++) {
                vars.add(arr.getString(i));
            }
        }
        return Collections.unmodifiableList(vars);
    }

    private static List<Map<String, Object>> parseRows(JSONObject data) {
        JSONArray rowsArray = data.optJSONArray("rows");
        if (rowsArray == null) {
            return List.of();
        }
        List<Map<String, Object>> result = new ArrayList<>(rowsArray.length());
        for (int i = 0; i < rowsArray.length(); i++) {
            JSONObject rowObj = rowsArray.getJSONObject(i);
            JSONObject values = rowObj.optJSONObject("values");
            if (values == null) {
                result.add(Map.of());
                continue;
            }
            Map<String, Object> row = new HashMap<>();
            for (String key : values.keySet()) {
                row.put(key, unwrap(values.get(key)));
            }
            result.add(Map.copyOf(row));
        }
        return List.copyOf(result);
    }

    private static Object unwrap(Object value) {
        if (!(value instanceof JSONObject obj)) {
            return toJavaNumber(value);
        }
        if (obj.length() == 1) {
            String tag = obj.keys().next();
            Object inner = obj.get(tag);
            if (inner instanceof JSONArray arr) {
                List<Object> list = new ArrayList<>(arr.length());
                for (int i = 0; i < arr.length(); i++) {
                    list.add(unwrap(arr.get(i)));
                }
                return list;
            }
            return unwrap(inner);
        }
        return obj;
    }

    private static Object toJavaNumber(Object value) {
        if (value instanceof Number n) {
            double d = n.doubleValue();
            if (d == Math.floor(d) && !Double.isInfinite(d)) {
                if (d >= Integer.MIN_VALUE && d <= Integer.MAX_VALUE) {
                    return (int) d;
                }
                return (long) d;
            }
            return d;
        }
        return value;
    }

    public List<String> getVariables() {
        return variables;
    }

    public List<Map<String, Object>> getRows() {
        return rows;
    }

    public int getRowCount() {
        return rows.size();
    }

    public Optional<Map<String, Object>> first() {
        return rows.isEmpty() ? Optional.empty() : Optional.of(rows.get(0));
    }

    public List<Object> column(String columnName) {
        List<Object> values = new ArrayList<>();
        for (Map<String, Object> row : rows) {
            values.add(row.get(columnName));
        }
        return values;
    }

    public boolean isEmpty() {
        return rows.isEmpty();
    }

    @Override
    public String toString() {
        return String.format("QueryResult(rows=%d, variables=%s)", rows.size(), variables);
    }
}
