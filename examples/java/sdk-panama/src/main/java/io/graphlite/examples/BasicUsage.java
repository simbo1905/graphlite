package io.graphlite.examples;

import io.graphlite.sdk.*;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

/**
 * Basic usage example for GraphLite Java SDK (Panama/FFM).
 */
public class BasicUsage {

    public static void main(String[] args) {
        System.out.println("=== GraphLite Java SDK (Panama) Basic Usage ===\n");

        try {
            Path tempDir = Files.createTempDirectory("graphlite_java_");
            System.out.println("Using temporary database: " + tempDir + "\n");

            runExample(tempDir.toString());
            System.out.println("\n=== Example completed successfully ===");

        } catch (GraphLiteException e) {
            System.err.println("\nGraphLite Error: " + e.getMessage());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("\nUnexpected error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static void runExample(String dbPath) throws Exception {
        try (GraphLite db = GraphLite.open(dbPath)) {
            System.out.println("1. GraphLite version: " + GraphLite.version() + "\n");

            try (Session session = db.session("admin")) {
                System.out.println("2. Session created\n");

                System.out.println("3. Setting up schema and graph...");
                session.execute("CREATE SCHEMA IF NOT EXISTS /example");
                session.execute("SESSION SET SCHEMA /example");
                session.execute("CREATE GRAPH IF NOT EXISTS social");
                session.execute("SESSION SET GRAPH social");
                System.out.println("   Schema and graph created\n");

                System.out.println("4. Inserting data...");
                session.execute("INSERT (:Person {name: 'Alice', age: 30})");
                session.execute("INSERT (:Person {name: 'Bob', age: 25})");
                session.execute("INSERT (:Person {name: 'Charlie', age: 35})");
                System.out.println("   Inserted 3 persons\n");

                System.out.println("5. Querying: All persons");
                QueryResult result = session.query("MATCH (p:Person) RETURN p.name as name, p.age as age");
                System.out.println("   Found " + result.getRowCount() + " persons:");
                for (Map<String, Object> row : result.getRows()) {
                    System.out.println("   - " + row.get("name") + ": " + row.get("age") + " years old");
                }
                System.out.println();

                System.out.println("6. Filtering: Persons older than 25");
                result = session.query("MATCH (p:Person) WHERE p.age > 25 RETURN p.name as name, p.age as age ORDER BY p.age ASC");
                System.out.println("   Found " + result.getRowCount() + " persons over 25:");
                for (Map<String, Object> row : result.getRows()) {
                    System.out.println("   - " + row.get("name") + ": " + row.get("age") + " years old");
                }
                System.out.println();

                System.out.println("7. Aggregation");
                result = session.query("MATCH (p:Person) RETURN count(p) as total, avg(p.age) as avg_age");
                result.first().ifPresent(row -> {
                    System.out.println("   Total persons: " + row.get("total"));
                    Object avgAge = row.get("avg_age");
                    System.out.println("   Average age: " + (avgAge instanceof Number n ? n.doubleValue() : avgAge));
                });
                System.out.println();

                System.out.println("8. Column extraction");
                result = session.query("MATCH (p:Person) RETURN p.name as name");
                System.out.println("   All names: " + result.column("name"));
            }
        }
    }
}
