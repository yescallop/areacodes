import java.io.File
import java.io.Writer
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardOpenOption

const val DATA_DIRECTORY = "data"
const val RESULT_FILENAME = "result.csv"
const val CSV_HEADER = "代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间,弃用时间"
val allMap = HashMap<Int, Area>(8000)
val bw: Writer = Files.newBufferedWriter(
        Path.of(RESULT_FILENAME),
        StandardCharsets.UTF_8,
        StandardOpenOption.CREATE,
        StandardOpenOption.TRUNCATE_EXISTING
).append('\uFEFF').append(CSV_HEADER).append('\n')

fun main() {
    val curMap = HashMap<Int, String>(4000)

    File(DATA_DIRECTORY).listFiles()!!.forEach { file ->
        val curTime = file.name.toString()
                .substringBeforeLast('.').toInt()
        curMap.clear()
        file.forEachLine {
            curMap[it.substring(0, 6).toInt()] = it.substring(7)
        }
        allMap.filter { !curMap.contains(it.key) && !it.value.deprecated }
                .values.forEach { area ->
            area.entries.add(Entry(curTime, ""))
            area.deprecated = true
        }
        curMap.forEach { (code, name) ->
            val area = allMap[code]
            if (area != null) {
                if (area.entries.last().name != name)
                    area.entries.add(Entry(curTime, name))
                area.deprecated = false
            } else allMap[code] = Area(curTime, name)
        }
        println("Processed: $curTime")
    }

    allMap.toSortedMap().forEach { (code, area) ->
        val entries = area.entries
        val size = entries.size
        val last = size - if (area.deprecated) 2 else 1
        for (i in last downTo 0) {
            val name = entries[i].name
            if (name.isEmpty()) continue
            val isLast = i == last
            val end = if (i + 1 >= size) null else entries[i + 1].time
            writeEntry(code, name, entries[i].time, end, isLast)
        }
    }
    bw.close()
}

fun writeEntry(code: Int, name: String, start: Int, end: Int?, isLast: Boolean) {
    val level = levelFromCode(code)

    val primary = allMap[code / 10000 * 10000]!!.entries[0].name
    val secondary = when (level) {
        Level.PREFECTURE -> name
        Level.COUNTY -> {
            val secondaryArea = allMap[code / 100 * 100]
            when {
                secondaryArea == null -> "直管"
                end == null -> secondaryArea.entries.last().name
                else -> secondaryArea.lastNameIntersecting(start, end) ?: "直管"
            }
        }
        Level.PROVINCE -> ""
    }

    val status = when {
        end == null -> "启用"
        isLast -> "弃用"
        else -> "变更"
    }
    //代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间,弃用时间
    bw.write(String.format("%d,%s,%s,%s,%s,%s,%d,%s\n",
            code, primary, secondary, name, level.desc,
            status, start, end ?: ""
    ))
    bw.flush()
}

fun levelFromCode(code: Int) = when {
    code % 100 != 0 -> Level.COUNTY
    code % 10000 != 0 -> Level.PREFECTURE
    else -> Level.PROVINCE
}

enum class Level(val desc: String) {
    PROVINCE("省级"),
    PREFECTURE("地级"),
    COUNTY("县级");
}

class Area(time: Int, name: String) {
    val entries = ArrayList<Entry>(1)
            .apply { add(Entry(time, name)) }
    var deprecated = false

    fun lastNameIntersecting(start: Int, end: Int): String? {
        val last = entries.size - 1
        for (i in last downTo 0) {
            val cur = entries[i]
            if (i == last && !deprecated) {
                if (cur.time < end)
                    return cur.name
                continue
            }
            if (cur.name.isEmpty()) continue

            // check if P=[curTime,nextTime) and Q=[start,end) intersect
            // we have 2 propositions
            // A: P is left separate from Q (nextTime <= start)
            // B: P is right separate from Q (curTime >= end)
            // then just !(A || B) <=> !A && !B
            if (entries[i + 1].time > start && cur.time < end)
                return cur.name
        }
        return null
    }
}

class Entry(val time: Int, val name: String)