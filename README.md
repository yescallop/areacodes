# areacodes

这个项目收集并分析**中华人民共和国县级以上行政区划代码**自 1980 年至今的历史数据及新旧代码间的对应关系数据。

## 在线预览

[由此进入在线预览][preview]

## 数据集情况

| 数据集名称 | 规范 | 目录 | 汇总 | 进度 |
| - | - | - | - | - |
| 历史行政区划代码 | 无 | [data](data) | [JSON] \| [CSV] | 100% |
| 新旧代码对应关系 | [diff 规范](diff-spec.md) | [diff](diff) | 同上 | 100% |

## CSV 汇总表说明

- 启用 / 弃用时间只精确到年。

    注：自 2018 年起至 2020 年底，民政部发布的数据每月更新一次，但自 2021 年起又改为每年更新一次。为了维持一致性，汇总表中记录的时间只精确到年。
- 一条记录若为省级，其“二级行政区”为空；若为地级，其“二级行政区”即为其名称；若为县级，其“二级行政区”为其启用时对应的上级行政区名称（若上级代码存在），或“直辖”（若上级代码不存在）。

    注：先前的版本中“直辖”原为“直管”。
- 一条记录的“新代码”为代码变更、弃用或行政区域变更后，原行政区域所对应的代码（若该记录的状态为“启用”，则“新代码”不含其自身的代码）。

    注：“新代码”中代码以字符 `;` 分隔。若一代码后接包含年份的方括号（如 `[2010]`），说明其对应的行政区域变更发生在指定的年份，否则默认为“变更（弃用）时间”。
- 查询一条记录对应的最新代码的方法是，以该记录为根节点，按“新代码”字段展开树节点，直至所有叶节点的“新代码”字段均为空且状态均为“启用”为止。
- 在行政区划合并后又拆分的情况（或其他类似情况）下，按上述方法或不能精确查询一条记录对应的最新代码。后续可通过扩展语法解决此问题。
- 本汇总表适用 [CC0] 许可协议，仅供参考之用，不建议用于其他用途。

## 原始数据来源

- [中华人民共和国县级以上行政区划代码][1] ([2018][1.1], [2019][1.2], [2020][1.3])，民政部
- [县级以上行政区划变更情况][2]，民政部
- [中华人民共和国行政区划沿革][3]，中国政府网

[preview]: https://yescallop.cn/areacodes/
[CSV]: https://raw.githubusercontent.com/yescallop/areacodes/master/result.csv
[JSON]: https://raw.githubusercontent.com/yescallop/areacodes/master/codes.json
[1]: http://www.mca.gov.cn/article/sj/xzqh/1980/
[1.1]: http://www.mca.gov.cn/article/sj/xzqh/2018/
[1.2]: http://www.mca.gov.cn/article/sj/xzqh/2019/
[1.3]: http://www.mca.gov.cn/article/sj/xzqh/2020/
[2]: http://xzqh.mca.gov.cn/description?dcpid=1
[3]: http://www.gov.cn/test/2006-02/27/content_212020.htm
[CC0]: https://creativecommons.org/publicdomain/zero/1.0/deed.zh
