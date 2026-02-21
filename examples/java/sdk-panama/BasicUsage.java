import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.Session;
import io.graphlite.sdk.QueryResult;
import io.graphlite.sdk.Errors.*;

import java.io.IOException;
import java.nio.file.*;
import java.util.Comparator;
import java.util.Map;

/**
 * Basic usage example for the GraphLite Java SDK (Panama FFM).
 * <p>
 * Demonstrates the core SDK API: open, session, execute, query, close.
 *
 * Run with:
 * <pre>
 * mvn compile exec:java -Dexec.mainClass="BasicUsage"
 * </pre>
 */
public class BasicUsage {

    public static void main(String[] args) {
        System.out.println("=== GraphLite Java SDK (Panama FFM) Basic Usage ===\n");

        Path tempDir;
        try {
            tempDir = Files.createTempDirectory("graphlite_basic_");
        } catch (IOException e) {
            System.err.println("Failed to create temp directory: " + e);
            return;
        }

        try {
            runExample(tempDir.toString());
            System.out.println("\n=== Example completed successfully ===");
        } catch (GraphLiteException e) {
            System.err.println("\n[ERROR] " + e.errorCode() + ": " + e.getMessage());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("\n[ERROR] Unexpected: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        } finally {
            try { deleteDir(tempDir); } catch (IOException ignored) {}
        }
    }

    private static void runExample(String dbPath) {
        // 1. Open database (AutoCloseable)
        System.out.println("1. Opening database...");
        try (GraphLite db = GraphLite.open(dbPath)) {
            System.out.println("   [OK] GraphLite version: " + GraphLite.version() + "\n");

            // 2. Create session (AutoCloseable, session-centric API)
            System.out.println("2. Creating session...");
            try (Session session = db.session("admin")) {
                System.out.println("   [OK] Session for '" + session.username() + "'\n");

                // 3. Schema and graph setup
                System.out.println("3. Setting up schema and graph...");
                session.execute("CREATE SCHEMA IF NOT EXISTS /example");
                session.execute("SESSION SET SCHEMA /example");
                session.execute("CREATE GRAPH IF NOT EXISTS social");
                session.execute("SESSION SET GRAPH social");
                System.out.println("   [OK] Schema and graph created\n");

                // 4. Insert data
                System.out.println("4. Inserting data...");
                session.execute("INSERT (:Person {name: 'Alice', age: 30})");
                session.execute("INSERT (:Person {name: 'Bob', age: 25})");
                session.execute("INSERT (:Person {name: 'Charlie', age: 35})");
                System.out.println("   [OK] Inserted 3 persons\n");

                // 5. Query all
                System.out.println("5. Querying all persons...");
                QueryResult result = session.query(
                    "MATCH (p:Person) RETURN p.name AS name, p.age AS age");
                System.out.println("   Found " + result.rowCount() + " persons:");
                for (Map<String, Object> row : result.rows()) {
                    System.out.println("   - " + row.get("name") + ": " + row.get("age") + " years old");
                }
                System.out.println();

                // 6. Filter with WHERE
                System.out.println("6. Filtering: age > 25...");
                result = session.query(
                    "MATCH (p:Person) WHERE p.age > 25 " +
                    "RETURN p.name AS name, p.age AS age ORDER BY p.age ASC");
                System.out.println("   Found " + result.rowCount() + " persons over 25:");
                for (Map<String, Object> row : result.rows()) {
                    System.out.println("   - " + row.get("name") + ": " + row.get("age"));
                }
                System.out.println();

                // 7. Aggregation
                System.out.println("7. Aggregation query...");
                result = session.query(
                    "MATCH (p:Person) RETURN count(p) AS total, avg(p.age) AS avg_age");
                if (!result.isEmpty()) {
                    Map<String, Object> row = result.first();
                    System.out.println("   Total: " + row.get("total"));
                    System.out.println("   Average age: " + row.get("avg_age"));
                }
                System.out.println();

                // 8. Column extraction
                System.out.println("8. Extracting column values...");
                result = session.query("MATCH (p:Person) RETURN p.name AS name");
                System.out.println("   All names: " + result.column("name"));
            } // Session auto-closed
        } // Database auto-closed
        System.out.println("\n9. Database closed");
    }

    private static void deleteDir(Path dir) throws IOException {
        if (!Files.exists(dir)) return;
        try (var walk = Files.walk(dir)) {
            walk.sorted(Comparator.reverseOrder()).forEach(p -> {
                try { Files.deleteIfExists(p); } catch (IOException ignored) {}
            });
        }
    }
}
