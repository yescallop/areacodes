import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import java.util.function.Consumer;

/**
 * @author Scallop Ye
 */
public class Main {
    public static void main(String[] args) {
        Map<Integer, Area> map = new HashMap<>();
        Map<Integer, String> each = new HashMap<>();
        Consumer<Path> processor = file -> {
            String name = file.getFileName().toString();
            final int year = Integer.parseInt(name.substring(0, name.lastIndexOf('.'))); //cut the file name
            each.clear();
            try {
                //parse the file into a map
                Files.lines(file).forEach(line -> each.put(Integer.parseInt(line.substring(0, 6)), line.substring(7)));
            } catch (IOException e) {
                e.printStackTrace();
                System.exit(1);
                return;
            }
            map.entrySet().stream()
                    .filter(e -> !each.containsKey(e.getKey()) && !e.getValue().ended)
                    .forEach(e -> {
                        Area a = e.getValue();
                        a.time.add(year);
                        a.ended = true;
                        a.names.add("-");
                    });
            each.forEach((k, v) -> {
                if (map.containsKey(k)) {
                    Area a = map.get(k);
                    if (a.ended)
                        a.ended = false;
                    if (!a.names.get(a.names.size() - 1).equals(v)) {
                        a.names.add(v);
                        a.time.add(year);
                    }
                } else
                    map.put(k, new Area(v, year));
            });
            System.out.println("Processed: " + year);
        };

        try {
            Files.list(Paths.get("data"))
                    .filter(Files::isRegularFile)
                    .forEach(processor);
        } catch (IOException e) {
            e.printStackTrace();
        }

        int[] codes = map.keySet().stream().mapToInt(Integer::intValue).sorted().toArray();
        for (int c : codes) {
            Area a = map.get(c);
            System.out.printf("%d\t%s\t%s\n", c, a.names, a.time);
        }
    }

    private static class Area {

        List<String> names = new LinkedList<>();
        Set<Integer> time = new LinkedHashSet<>();
        boolean ended = false;

        Area(String name, int start) {
            this.names.add(name);
            this.time.add(start);
        }
    }
}
