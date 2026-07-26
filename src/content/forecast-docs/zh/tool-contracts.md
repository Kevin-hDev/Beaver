# Forecast 工具

七个 Forecast 工具组成受控流程。大型结果保存在 Forecast 存储中，LLM 主要交换紧凑的标识符。

## 推荐顺序

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

之后使用 `forecast_analyze` 处理笔记、场景或模型组合。

## `forecast_data_audit`

每个新数据集第一次预测前，请调用此工具。请提供数据或文件、目标、日期、频率、范围和精确置信水平。

它会验证日期、重复项、缺失周期、无效值、历史长度、序列、未来行和异常值。有效响应会返回 `data_profile_id`。

## `forecast_models`

请检查当前策略和区间能力。手动模式中检查强制模型；自动模式中传入 `data_profile_id`，选择一个候选并保留 `selection_id`。

硬件信息只会在此 Forecast 响应中出现。不要对置信水平进行四舍五入。

## `forecast`

请使用档案、目标、日期、范围、频率和不变的置信水平运行预测。只有模型支持时才添加序列和协变量。

自动模式还需传入模型、`selection_id`、选择来源和允许的理由。响应会返回 `analysis_id`。

## `forecast_read`

省略 `analysis_id` 可列出有限数量的分析；提供它可读取指定分析。使用 `offset` 和 `limit` 分页，每页最多 200 点。

读取结果还可能包含分解、残差异常、按时间顺序的置换重要性和漂移。缺失时不要编造替代结果。

## `forecast_backtest`

请对已保存分析运行受限的滚动时间验证。模型以及 Naive、Seasonal Naive、Drift 和 ETS 基准会在相同周期上评估。

请始终检查状态和失败项。

## `forecast_compare_models`

请读取已保存排名，包括误差、覆盖率、耗时、观测内存和基准状态。只有完整可比结果支持时，才可以称某模型最佳。

## `forecast_analyze`

请使用 `annotate`、`scenario`、`scenario_update`、`scenario_delete` 或 `ensemble`。只有多模型回测成功后才创建组合，并说明它按 MASE 倒数加权且尚未独立评估。

## 重新开始流程

数据、映射、目标、频率、范围、置信水平、协变量、序列结构或资源发生变化时，请重新调用 `forecast_data_audit` 和 `forecast_models`。
