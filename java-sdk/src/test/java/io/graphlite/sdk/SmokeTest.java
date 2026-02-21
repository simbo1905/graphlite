package io.graphlite.sdk;

import java.io.IOException;
import java.nio.file.*;
import java.util.Comparator;
import java.util.Map;

/**
 * Minimal smoke test that can run as a standalone {@code main()} or as a JUnit test.
 * <p>
 * Verifies: open &rarr; session &rarr; schema/graph setup &rarr; insert &rarr; query
 * &rarr; validate row count &rarr; close all resources without crashes.
 */
public class SmokeTest {

    public static void main(String[] args) throws Exception {
        Path tmpDir = Files.createTempDirectory("graphlite_smoke_");
        try {
            runSmoke(tmpDir.toString());
            System.out.println("SMOKE TEST PASSED");
        } finally {
            deleteRecursively(tmpDir);
        }
    }

    static void runSmoke(String dbPath) {
        try (GraphLite db = GraphLite.open(dbPath)) {
            String ver = GraphLite.version();
            assert ver != null && !ver.isBlank() : "version must not be blank";
            System.out.println("  version: " + ver);

            try (Session session = db.session("tester")) {
                assert "tester".equals(session.username());
                assert session.id() != null && !session.id().isBlank();

                session.execute("CREATE SCHEMA IF NOT EXISTS /smoke");
                session.execute("SESSION SET SCHEMA /smoke");
                session.execute("CREATE GRAPH IF NOT EXISTS g");
                session.execute("SESSION SET GRAPH g");

                session.execute("INSERT (:Item {name: 'A', value: 1})");
                session.execute("INSERT (:Item {name: 'B', value: 2})");
                session.execute("INSERT (:Item {name: 'C', value: 3})");

                QueryResult result = session.query(
                    "MATCH (i:Item) RETURN i.name AS name, i.value AS value ORDER BY i.value ASC"
                );

                assert result.rowCount() == 3 : "expected 3 rows, got " + result.rowCount();
                assert !result.isEmpty();

                Map<String, Object> first = result.first();
                assert "A".equals(first.get("name")) : "first row name should be A";

                assert result.variables().contains("name");
                assert result.variables().contains("value");

                assert result.column("name").size() == 3;

                System.out.println("  rows: " + result.rows());
            }
        }
    }

    private static void deleteRecursively(Path dir) throws IOException {
        if (!Files.exists(dir)) return;
        try (var walk = Files.walk(dir)) {
            walk.sorted(Comparator.reverseOrder()).forEach(p -> {
                try { Files.deleteIfExists(p); } catch (IOException ignored) {}
            });
        }
    }
}
