# areacodes

这个项目收集并分析**中华人民共和国县级以上行政区划代码**自 1980 年至今的历史数据及新旧代码间的对应关系数据。

项目内的工具均使用 Rust 编写，原先的 Kotlin 代码现已不再维护。

## 数据集情况

| 数据集名称 | 规范 | 目录 | 汇总 | 进度 |
| - | - | - | - | - |
| 历史行政区划代码 | 无 | [data](data) | [CSV](result.csv) | 100% |
| 新旧代码对应关系 | [diff 规范](diff-spec.md) | [diff](diff) | 暂无 | 71.1% |

## 原始数据来源

- [中华人民共和国县级以上行政区划代码][1] ([2018][1.1], [2019][1.2], [2020][1.3])，民政部
- [县级以上行政区划变更情况][2]，民政部
- [中华人民共和国行政区划沿革][3]，中国政府网

[1]: http://www.mca.gov.cn/article/sj/xzqh/1980/
[1.1]: http://www.mca.gov.cn/article/sj/xzqh/2018/
[1.2]: http://www.mca.gov.cn/article/sj/xzqh/2019/
[1.3]: http://www.mca.gov.cn/article/sj/xzqh/2020/
[2]: http://xzqh.mca.gov.cn/description?dcpid=1
[3]: http://www.gov.cn/test/2006-02/27/content_212020.htm
