import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.QueryResult;
import io.graphlite.sdk.Session;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.Map;

/**
 * Minimal GraphLite Java SDK (Panama) example.
 */
public final class BasicUsage {
    private BasicUsage() {
    }

    public static void main(String[] args) throws IOException {
        Path dbPath = Files.createTempDirectory("graphlite-panama-basic-");
        try (GraphLite db = GraphLite.open(dbPath.toString());
             Session session = db.session("admin")) {
            session.execute("CREATE SCHEMA IF NOT EXISTS /example");
            session.execute("SESSION SET SCHEMA /example");
            session.execute("CREATE GRAPH IF NOT EXISTS social");
            session.execute("SESSION SET GRAPH social");

            session.execute("INSERT (:Person {name: 'Alice', age: 30})");
            session.execute("INSERT (:Person {name: 'Bob', age: 25})");
            session.execute("INSERT (:Person {name: 'Charlie', age: 35})");

            QueryResult result = session.query(
                "MATCH (p:Person) RETURN p.name AS name, p.age AS age ORDER BY p.age ASC"
            );

            System.out.println("People in graph:");
            for (Map<String, Object> row : result) {
                System.out.println(" - " + row.get("name") + " (" + row.get("age") + ")");
            }
            System.out.println("All names via column helper: " + result.column("name"));
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
                        // Best-effort cleanup for temporary example directories.
                    }
                });
        }
    }
}
