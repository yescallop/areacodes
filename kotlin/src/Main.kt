import java.io.File
import java.io.Writer
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.StandardOpenOption

const val DATA_DIRECTORY = "data"
const val RESULT_FILENAME = "result.csv"
const val CSV_HEADER = "\uFEFF代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间（含）,弃用时间（不含）\n"
private val allMap = HashMap<Int, Area>(8192, 1f)
private val bw: Writer = Files.newBufferedWriter(
    Paths.get(RESULT_FILENAME),
    StandardCharsets.UTF_8,
    StandardOpenOption.CREATE,
    StandardOpenOption.TRUNCATE_EXISTING
).append(CSV_HEADER)

fun main() {
    val curMap = HashMap<Int, String>(4096, 1f)

    File(DATA_DIRECTORY).listFiles()!!.forEach { file ->
        val curTime = file.name.toString()
            .substringBeforeLast('.').toInt()
        curMap.clear()
        file.forEachLine {
            curMap[it.substring(0, 6).toInt()] = it.substring(7)
        }
        allMap.forEach { (code, area) ->
            if (code !in curMap && !area.deprecated) {
                area.entries.add(Entry(curTime, ""))
                area.deprecated = true
            }
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
            val entry = entries[i]
            val name = entry.name
            if (name.isEmpty()) continue
            val end = entries.getOrNull(i + 1)?.time
            writeEntry(code, name, entry.time, end, i == last)
        }
    }
    bw.close()
}

fun writeEntry(code: Int, name: String, start: Int, end: Int?, isLast: Boolean) {
    val level = levelFromCode(code)

    val province = allMap[code / 10000 * 10000]!!.entries[0].name
    val prefecture = when (level) {
        Level.PREFECTURE -> name
        Level.COUNTY -> allMap[code / 100 * 100]
            ?.lastNameIntersecting(start, end) ?: "直管"
        Level.PROVINCE -> ""
    }

    val status = when {
        end == null -> "启用"
        isLast -> "弃用"
        else -> "变更"
    }
    //代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间,弃用时间
    bw.write(
        "$code,$province,$prefecture,$name," +
                "${level.desc},$status,$start,${end ?: ""}\n"
    )
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

class Area(code: Int, name: String) {
    val entries = ArrayList<Entry>(1).apply { add(Entry(code, name)) }
    var deprecated = false

    fun lastNameIntersecting(start: Int, end: Int?): String? {
        val last = entries.lastIndex
        if (end == null) return entries[last].name
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