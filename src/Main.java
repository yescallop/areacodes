import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;

/**
 * @author Scallop Ye
 */
public class Main {

    public static void main(String[] args) {
        Map<Integer, Area> map = new ConcurrentHashMap<>();
        Map<Integer, String> each = new HashMap<>();
        for (int year = 1980; year <= 201807; year++) {
            try {
                BufferedReader reader = new BufferedReader(new FileReader(year + ".txt"));
                each.clear();
                String line;
                while ((line = reader.readLine()) != null) {
                    int code = Integer.parseInt(line.substring(0, 6));
                    String name = line.substring(7);
                    each.put(code, name);
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
            } catch (IOException e) {
                e.printStackTrace();
            }
            System.out.println("Processed: " + year);
            if (year == 2017) year = 201800;
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
