# areacodes

本项目收集**中华人民共和国县级以上行政区划代码**自 1984 年至今的历史数据及新旧代码间的对应关系数据。

## 在线预览

[由此进入在线预览](https://yescallop.github.io/areacodes/)

## 数据集情况

| 数据集名称 | 规范 | 目录 | 汇总 | 进度 |
| - | - | - | - | - |
| 历史行政区划代码 | `代码 名称` | [data](data) | [JSON] \| [CSV] | 已完成 |
| 新旧代码对应关系 | [diff 规范](diff-spec.md) | [diff](diff) | 同上 | 基本完成* |

\* 区划变更的文本描述收集工作现已开始，欢迎各位参与，详情参见 [Issue #6](https://github.com/yescallop/areacodes/issues/6)。

## CSV 汇总表说明

- 各年数据截至当年 12 月 31 日。一条记录的在用时间段为启用时间（含）至变更/弃用时间（不含）。
- 一条记录若为省级，其“二级行政区”为空；若为地级，其“二级行政区”即为其名称；若为县级，其“二级行政区”为其在用时对应的上级行政区名称（若上级代码存在）或“直辖”（若上级代码不存在）。
- 一条记录的“新代码”为代码变更、弃用或行政区域变更后，原行政区域所对应的代码（若该记录的状态为“在用”，则为行政区域变更，此时“新代码”不含其自身的代码）。多个代码以字符 `;` 分隔。若一代码后接包含年份的方括号（如 `[2010]`），说明其对应的行政区域变更发生在指定的年份，否则默认为“变更/弃用时间”。

## 许可协议

本项目适用 [CC0 1.0] 许可协议。

## 原始数据来源

- [中华人民共和国县级以上行政区划代码][1]（[最新数据][1a]），民政部。
- [县级以上行政区划变更情况][2]，民政部。
- [国务院公报][3]，中国政府网。
- [全国行政区划信息查询平台][4]，民政部。
- [原始数据勘误](errata.md)，本项目。

[CSV]: https://raw.githubusercontent.com/yescallop/areacodes/master/result.csv
[JSON]: https://raw.githubusercontent.com/yescallop/areacodes/master/codes.json
[CC0 1.0]: https://creativecommons.org/publicdomain/zero/1.0/deed.zh-hans
[1]: https://www.mca.gov.cn/n156/n186/index.html
[1a]: https://www.mca.gov.cn/n156/n2679/index.html
[2]: http://xzqh.mca.gov.cn/description?dcpid=1
[3]: https://www.gov.cn/gongbao/
[4]: http://xzqh.mca.gov.cn/map
