package io.graphlite.sdk;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import io.graphlite.sdk.Errors.SerializationException;

import java.util.*;

/**
 * Immutable wrapper around the JSON result returned by the native
 * {@code graphlite_query} function.
 * <p>
 * The JSON format is:
 * <pre>{@code
 * {
 *   "variables": ["col1", "col2"],
 *   "rows": [
 *     {"values": {"col1": {"String": "v1"}, "col2": {"Number": 42}}},
 *     ...
 *   ],
 *   "row_count": 2
 * }
 * }</pre>
 * <p>
 * Serde-encoded enum wrappers like {@code {"String": "v"}} are unwrapped
 * automatically so callers see plain Java objects.
 */
public final class QueryResult {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    private final List<String> variables;
    private final List<Map<String, Object>> rows;

    QueryResult(String json) {
        try {
            JsonNode root = MAPPER.readTree(json);
            this.variables = parseVariables(root);
            this.rows = parseRows(root);
        } catch (Exception e) {
            throw new SerializationException("Failed to parse query result JSON", e);
        }
    }

    /** Column/variable names from the RETURN clause. */
    public List<String> variables() {
        return variables;
    }

    /** All result rows. Each row is an unmodifiable {@code Map<String,Object>}. */
    public List<Map<String, Object>> rows() {
        return rows;
    }

    /** Number of rows. */
    public int rowCount() {
        return rows.size();
    }

    /** First row, or {@code null} if the result is empty. */
    public Map<String, Object> first() {
        return rows.isEmpty() ? null : rows.getFirst();
    }

    /** {@code true} if the result contains zero rows. */
    public boolean isEmpty() {
        return rows.isEmpty();
    }

    /**
     * Extract all values from a single column across all rows.
     *
     * @param columnName the column to extract
     * @return list of values (may contain {@code null} for missing columns)
     */
    public List<Object> column(String columnName) {
        List<Object> values = new ArrayList<>(rows.size());
        for (Map<String, Object> row : rows) {
            values.add(row.get(columnName));
        }
        return values;
    }

    @Override
    public String toString() {
        return "QueryResult(rows=" + rows.size() + ", variables=" + variables + ")";
    }

    // --- JSON parsing internals ---

    private static List<String> parseVariables(JsonNode root) {
        List<String> vars = new ArrayList<>();
        JsonNode arr = root.path("variables");
        if (arr.isArray()) {
            for (JsonNode v : arr) vars.add(v.asText());
        }
        return Collections.unmodifiableList(vars);
    }

    private static List<Map<String, Object>> parseRows(JsonNode root) {
        JsonNode arr = root.path("rows");
        if (!arr.isArray()) return List.of();

        List<Map<String, Object>> result = new ArrayList<>(arr.size());
        for (JsonNode rowNode : arr) {
            JsonNode values = rowNode.path("values");
            if (values.isMissingNode() || !values.isObject()) continue;

            Map<String, Object> row = new LinkedHashMap<>();
            Iterator<String> fieldNames = values.fieldNames();
            while (fieldNames.hasNext()) {
                String key = fieldNames.next();
                row.put(key, unwrap(values.get(key)));
            }
            result.add(Collections.unmodifiableMap(row));
        }
        return Collections.unmodifiableList(result);
    }

    /**
     * Unwrap serde-style tagged values ({@code {"String": "v"}}, {@code {"Number": 42}}).
     * If the node is a single-key object, recurse into the value.
     */
    private static Object unwrap(JsonNode node) {
        if (node == null || node.isNull()) return null;
        if (node.isTextual()) return node.asText();
        if (node.isInt()) return node.asInt();
        if (node.isLong()) return node.asLong();
        if (node.isDouble() || node.isFloat()) return node.asDouble();
        if (node.isBigDecimal()) {
            var bd = node.decimalValue();
            if (bd.scale() <= 0 && bd.compareTo(java.math.BigDecimal.valueOf(Long.MAX_VALUE)) <= 0
                    && bd.compareTo(java.math.BigDecimal.valueOf(Long.MIN_VALUE)) >= 0) {
                long lv = bd.longValueExact();
                if (lv >= Integer.MIN_VALUE && lv <= Integer.MAX_VALUE) return (int) lv;
                return lv;
            }
            return bd.doubleValue();
        }
        if (node.isBoolean()) return node.asBoolean();
        if (node.isArray()) {
            List<Object> list = new ArrayList<>(node.size());
            for (JsonNode child : node) list.add(unwrap(child));
            return list;
        }
        if (node.isObject() && node.size() == 1) {
            String tag = node.fieldNames().next();
            return unwrap(node.get(tag));
        }
        if (node.isObject()) {
            Map<String, Object> map = new LinkedHashMap<>();
            Iterator<String> fields = node.fieldNames();
            while (fields.hasNext()) {
                String key = fields.next();
                map.put(key, unwrap(node.get(key)));
            }
            return map;
        }
        return node.asText();
    }
}
