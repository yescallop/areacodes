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
            bwCsv.write("代码,一级行政区,二级行政区（弃用前）,名称,级别,状态,启用时间,弃用时间,是否自治,是否市辖区,是否县级市\n");

            BufferedWriter bwTxt = Files.newBufferedWriter(
                    Paths.get("result.txt"),
                    StandardCharsets.UTF_8,
                    StandardOpenOption.CREATE,
                    StandardOpenOption.TRUNCATE_EXISTING
            );

            int[] codes = map.keySet().stream().mapToInt(Integer::intValue).sorted().toArray();
            for (int c : codes) {
                Area a = map.get(c);

                int size = a.names.size();
                if (size == 1) {
                    writeArea(bwCsv, map, c, a.names.get(0), a.time.get(0), null, true);
                } else if (a.deprecated) {
                    for (int i = size - 2; i >= 0; i--) {
                        String name = a.names.get(i);
                        if (!name.equals("-"))
                            writeArea(bwCsv, map, c, name, a.time.get(i), a.time.get(i + 1), i == size - 2);
                    }
                } else {
                    for (int i = size - 1; i >= 0; i--) {
                        String name = a.names.get(i);
                        if (i == size - 1)
                            writeArea(bwCsv, map, c, name, a.time.get(i), null, true);
                        else if (!name.equals("-"))
                            writeArea(bwCsv, map, c, name, a.time.get(i), a.time.get(i + 1), i == size - 1);
                    }
                }

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

    private static void writeArea(BufferedWriter bw, Map<Integer, Area> map,
                                  int code, String name, int startTime, Integer endTime, boolean last) throws IOException {
        Level level = Level.fromCode(code);

        boolean isAutonomous = name.contains("自治");
        boolean isCountyLevelCity = (level == Level.COUNTY && name.endsWith("市"));
        boolean isDistrict = (level == Level.COUNTY && name.endsWith("区"));

        String primaryDistrict = map.get(code / 10000 * 10000).names.get(0);
        String secondaryDistrict = "";

        if (level == Level.PREFECTURE) {
            secondaryDistrict = name;
        } else if (level == Level.COUNTY) {
            Area cda = map.get(code / 100 * 100);
            if (cda == null) {
                secondaryDistrict = "直管";
            } else if (endTime == null) {
                secondaryDistrict = cda.lastName();
            } else {
                secondaryDistrict = cda.lastNameIntersecting(startTime, endTime);
                if (secondaryDistrict == null)
                    secondaryDistrict = "直管";
            }
        }

        //代码,一级行政区,二级行政区（弃用前）,名称,级别,状态,启用时间,弃用时间,是否自治,是否市辖区,是否县级市
        bw.write(String.format("%d,%s,%s,%s,%s,%s,%d,%s,%s,%s,%s\n",
                code, primaryDistrict, secondaryDistrict, name, level.description,
                endTime == null ? "启用" : (last ? "代码弃用" : "名称弃用"),
                startTime, endTime == null ? "" : endTime,
                isAutonomous ? "是" : "否", isDistrict ? "是" : "否", isCountyLevelCity ? "是" : "否"
        ));
        bw.flush();
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

        public String lastNameIntersecting(int start, int end) {
            int size = time.size();
            if (size == 1)
                return time.get(0) < end ? names.get(0) : null;

            for (int i = size - 1; i >= 0; i--) {
                int curTime = time.get(i);
                if (i == size - 1 && !deprecated) {
                    if (curTime < end)
                        return names.get(i);
                    continue;
                }

                String curName = names.get(i);
                if (curName.equals("-"))
                    continue;
                int nextTime = time.get(i + 1);
                // check if [curTime,nextTime) intercepts with [start,end)
                if ((start >= curTime && start < nextTime) || (curTime >= start && curTime < end))
                    return curName;
            }
            return null;
        }
    }
}
