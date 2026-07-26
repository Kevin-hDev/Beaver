# Forecast Tools

Forecast tools 让 LLM 代理可以在聊天中调用预测引擎。它们必须用精确的参数调用，因为列错误或预测期不一致会产生毫无意义的预测。

## `forecast`

`forecast` 启动一次新的预测。

主要输入：

| 参数 | 作用 |
| --- | --- |
| `file_path` | 要读取的 Excel、CSV 或 JSON 文件 |
| `data` | 已准备好的 JSON 数据 |
| `date_column` | 包含日期的列 |
| `target_column` | 需要预测的列 |
| `series_column` | 标识序列的列 |
| `covariate_columns` | 要使用的上下文变量 |
| `frequency` | 时间节奏 |
| `horizon` | 未来点数 |
| `model` | 要使用的引擎 |

主要输出：

- `analysis_id`，Forecast 结果的标识符。

## `forecast_read`

`forecast_read` 复查一份 Forecast 结果。

它用于取回：

- 预测；
- 历史；
- 不确定性；
- 场景；
- 可用变量；
- 模型元数据。

如果没有提供 `analysis_id`，代理可以用它列出可用结果。

## `forecast_analyze`

`forecast_analyze` 添加或修改预测周边的元素。

它尤其用于：

- 创建一条注释；
- 创建一个场景；
- 重新运行一个上下文场景；
- 修改一个场景；
- 删除一个场景。

## 代理应检查的内容

调用某个 tool 之前，代理必须检查：

- 目标存在；
- 日期可读；
- 预测期与未来行匹配；
- 协变量确实存在；
- 创建的或从网上找到的数据已标注来源；
- 所选模型支持该需求。

代理应当解释自己的选择，而不是发出一个不透明的调用。
