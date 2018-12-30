import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Consumer;

/**
 * @author Scallop Ye
 */
public class Main {

    public static void main(String[] args) {
        Map<Integer, Area> map = new ConcurrentHashMap<>();
        Map<Integer, String> each = new HashMap<>();
        Consumer<Path> processor = file -> {
            String name = file.getFileName().toString();
            int year = Integer.parseInt(name.substring(0, name.length() - 4));
            each.clear();
            try {
                Files.lines(file).forEach(line -> each.put(Integer.parseInt(line.substring(0, 6)), line.substring(7)));
            } catch (IOException e) {
                e.printStackTrace();
            }
            final int y = year;
            map.entrySet().parallelStream()
                    .filter(e -> !each.containsKey(e.getKey()) && !e.getValue().ended)
                    .forEach(e -> {
                        Area a = e.getValue();
                        a.time.add(y);
                        a.ended = true;
                        a.names.add("-");
                    });
            each.entrySet().parallelStream()
                    .forEach(e -> {
                        Integer k = e.getKey();
                        if (map.containsKey(k)) {
                            Area a = map.get(k);
                            if (a.ended)
                                a.ended = false;
                            if (!a.names.get(a.names.size() - 1).equals(e.getValue())) {
                                a.names.add(e.getValue());
                                a.time.add(y);
                            }
                        } else
                            map.put(k, new Area(e.getValue(), y));
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

        int[] codes = map.keySet().stream().mapToInt(a -> a).toArray();
        Arrays.sort(codes);
        for (int c : codes) {
            Area a = map.get(c);
            System.out.printf("%d\t%s\t%s", c, a.names, a.time);
            System.out.println();
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
