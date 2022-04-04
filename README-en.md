# areacodes (outdated)

This project collects and analyzes the historical data of **Administrative division codes of the People's Republic of China** (at and above county level) and the data of the correspondence between old and new codes.

The tools in the project are all written in Rust. The previous Kotlin code is no longer maintained.

## Dataset status

| Dataset name | Specification | Directory | Summary | Progress |
| - | - | - | - | - |
| Historical codes | N/A | [data](data) | [CSV](result.csv) | 100% |
| Code correspondence | [diff spec](diff-spec.md) | [diff](diff) | N/A | ~100% |

## Raw data source

- [Administrative division codes of the People's Republic of China (at and above county level)][1] ([2018][1.1], [2019][1.2], [2020][1.3]), Ministry of Civil Affairs.
- [Changes in administrative divisions (at and above county level)][2], Ministry of Civil Affairs.
- [History of the administrative divisions of the People's Republic of China][3], Portal of the State Council.

[1]: http://www.mca.gov.cn/article/sj/xzqh/1980/
[1.1]: http://www.mca.gov.cn/article/sj/xzqh/2018/
[1.2]: http://www.mca.gov.cn/article/sj/xzqh/2019/
[1.3]: http://www.mca.gov.cn/article/sj/xzqh/2020/
[2]: http://xzqh.mca.gov.cn/description?dcpid=1
[3]: http://www.gov.cn/test/2006-02/27/content_212020.htm
