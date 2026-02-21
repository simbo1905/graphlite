package io.graphlite.sdk;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Minimal smoke test: open temp db, create session, insert + query, validate row count.
 */
public class SmokeTest {

    public static void main(String[] args) {
        System.out.println("GraphLite Java SDK Smoke Test");
        try {
            Path tempDir = Files.createTempDirectory("graphlite_smoke_");
            try {
                runSmokeTest(tempDir.toString());
                System.out.println("PASS: Smoke test completed successfully");
            } finally {
                deleteRecursively(tempDir);
            }
        } catch (Exception e) {
            System.err.println("FAIL: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static void runSmokeTest(String dbPath) throws Exception {
        try (GraphLite db = GraphLite.open(dbPath)) {
            try (Session session = db.session("admin")) {
                session.execute("CREATE SCHEMA IF NOT EXISTS /test");
                session.execute("SESSION SET SCHEMA /test");
                session.execute("CREATE GRAPH IF NOT EXISTS g");
                session.execute("SESSION SET GRAPH g");
                session.execute("INSERT (:Node {id: 1, name: 'a'})");
                session.execute("INSERT (:Node {id: 2, name: 'b'})");

                QueryResult result = session.query("MATCH (n:Node) RETURN n.id, n.name");
                if (result.getRowCount() != 2) {
                    throw new AssertionError("Expected 2 rows, got " + result.getRowCount());
                }
                if (result.isEmpty()) {
                    throw new AssertionError("Result should not be empty");
                }
                result.first().orElseThrow(() -> new AssertionError("first() should return value"));
            }
        }
    }

    private static void deleteRecursively(Path path) throws java.io.IOException {
        if (Files.isDirectory(path)) {
            try (var stream = Files.list(path)) {
                stream.forEach(p -> {
                    try {
                        deleteRecursively(p);
                    } catch (java.io.IOException e) {
                        throw new RuntimeException(e);
                    }
                });
            }
        }
        Files.delete(path);
    }
}
