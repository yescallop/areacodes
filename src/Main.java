import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
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
                    .filter(e -> !each.containsKey(e.getKey()) && !e.getValue().deprecated)
                    .forEach(e -> {
                        Area a = e.getValue();
                        a.time.add(year);
                        a.deprecated = true;
                        a.names.add("-");
                    });
            each.forEach((k, v) -> {
                if (map.containsKey(k)) {
                    Area a = map.get(k);
                    if (a.deprecated)
                        a.deprecated = false;
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

        try {
            BufferedWriter bwCsv = Files.newBufferedWriter(
                    Paths.get("result.csv"),
                    StandardCharsets.UTF_8,
                    StandardOpenOption.CREATE,
                    StandardOpenOption.TRUNCATE_EXISTING
            );
            bwCsv.write('\ufeff'); //BOM
            bwCsv.write("代码,名称,级别,状态,变更时间,是否自治,是否市辖区,是否县级市,一级行政区,二级行政区,历史名称,历史变更时间,更多\n");

            BufferedWriter bwTxt = Files.newBufferedWriter(
                    Paths.get("result.txt"),
                    StandardCharsets.UTF_8,
                    StandardOpenOption.CREATE,
                    StandardOpenOption.TRUNCATE_EXISTING
            );

            int[] codes = map.keySet().stream().mapToInt(Integer::intValue).sorted().toArray();
            for (int c : codes) {
                Area a = map.get(c);
                String name = a.lastName();
                boolean isAutonomous = !a.deprecated && name.contains("自治");
                Level level = Level.fromCode(c);
                boolean isCountyLevelCity = !a.deprecated && level == Level.COUNTY && name.endsWith("市");
                boolean isDistrict = !a.deprecated && level == Level.COUNTY && name.endsWith("区");
                String primaryDistrict = map.get(c / 10000 * 10000).lastName();
                String secondaryDistrict = "";

                if (level != Level.PROVINCE && !a.deprecated) {
                    Area cda = map.get(c / 100 * 100);
                    if (cda == null || cda.deprecated) {
                        secondaryDistrict = "直管";
                    } else {
                        secondaryDistrict = cda.lastName();
                    }
                }

                bwCsv.write(String.format("%d,%s,%s,%s,%d,%s,%s,%s,%s,%s",
                        c, name, level.description, a.deprecated ? "弃用" : "使用", a.lastTime(),
                        isAutonomous ? "是" : "否", isDistrict ? "是" : "否", isCountyLevelCity ? "是" : "否",
                        primaryDistrict, secondaryDistrict
                ));

                int size = a.names.size();
                if (size != 1) {
                    for (int i = size - 2; i >= 0; i--) {
                        bwCsv.write(',' + a.names.get(i) + ',' + a.time.get(i));
                    }
                }
                bwCsv.write('\n');
                bwCsv.flush();

                bwTxt.write(String.format("%d\t%s\t%s\n", c, a.names, a.time));
                bwTxt.flush();
            }
            bwCsv.close();
            bwTxt.close();
        } catch (IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
    }

    private enum Level {
        PROVINCE("省级"),
        PREFECTURE("地级"),
        COUNTY("县级");

        String description;

        Level(String description) {
            this.description = description;
        }

        public static Level fromCode(int code) {
            if ((code % 100) != 0)
                return COUNTY;
            code /= 100;
            if ((code % 100) != 0)
                return PREFECTURE;
            return PROVINCE;
        }
    }

    private static class Area {
        List<String> names = new ArrayList<>();
        List<Integer> time = new ArrayList<>();
        boolean deprecated = false;

        Area(String name, int start) {
            this.names.add(name);
            this.time.add(start);
        }

        public String lastName() {
            return names.get(names.size() - 1);
        }

        public int lastTime() {
            return time.get(time.size() - 1);
        }
    }
}
