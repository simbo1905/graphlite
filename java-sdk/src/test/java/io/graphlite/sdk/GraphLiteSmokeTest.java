package io.graphlite.sdk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import org.junit.jupiter.api.Test;

class GraphLiteSmokeTest {

    @Test
    void openSessionInsertQueryAndClose() throws IOException {
        Path dbPath = Files.createTempDirectory("graphlite-panama-smoke-");
        try (GraphLite db = GraphLite.open(dbPath.toString());
             Session session = db.session("smoke-user")) {
            session.execute("CREATE SCHEMA IF NOT EXISTS /smoke");
            session.execute("SESSION SET SCHEMA /smoke");
            session.execute("CREATE GRAPH IF NOT EXISTS smoke_graph");
            session.execute("SESSION SET GRAPH smoke_graph");

            session.execute("INSERT (:Person {name: 'Alice', age: 30})");
            session.execute("INSERT (:Person {name: 'Bob', age: 25})");

            QueryResult result = session.query(
                "MATCH (p:Person) RETURN count(p) AS total"
            );

            assertFalse(result.isEmpty(), "Expected one row with aggregated count");
            assertEquals(1, result.rowCount(), "Expected a single aggregation row");

            Object totalValue = result.first().get("total");
            assertNotNull(totalValue, "Expected 'total' column in query result");
            assertEquals(2, ((Number) totalValue).intValue(), "Expected two inserted nodes");
        } finally {
            deleteRecursively(dbPath);
        }
    }

    private static void deleteRecursively(Path root) throws IOException {
        if (root == null || !Files.exists(root)) {
            return;
        }
        try (var walk = Files.walk(root)) {
            walk.sorted(Comparator.reverseOrder())
                .forEach(path -> {
                    try {
                        Files.deleteIfExists(path);
                    } catch (IOException ignored) {
                        // Best-effort cleanup for test temp directories.
                    }
                });
        }
    }
}
