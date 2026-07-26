# 模型

模型是计算预测的引擎。Forecast 提供多种模型家族，可以是本地的（计算留在本机），也可以是云端的（有用的数据会发送给已配置的提供商）。

## 本地家族

| 家族 | 发行方 | 说明 |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | 快速的本地模型，适合初次测试或简单目标 |
| TimesFM | Google | 用于时间序列预测的本地模型 |
| Toto 2.0 | Datadog | 面向监控和指标的本地模型 |
| MOIRAI 2.0 | Salesforce | 本地模型，支持多序列和协变量 |
| FlowState | IBM | 用于时间序列的本地模型 |
| TabPFN-TS | PriorLabs | 实验性本地模型 |
| TiRex | NX-AI | 实验性本地模型 |
| Kairos | Foundation Model Research | 实验性本地模型 |
| Sundial | THUML | 实验性本地模型 |

## 云端家族

| 家族 | 发行方 | 说明 |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | 专用于时间序列的云端引擎。需要 API 密钥，并将有用的数据发送给提供商。 |

云端模型可能更强大，但带来外部依赖和数据外发。对于敏感数据，优先选用本地模型。

## 选择模型

选择主要取决于数据集和使用场景：

- **快速测试、简单目标**：Chronos-Bolt。
- **敏感数据、本地计算**：任意本地家族。
- **协变量和未来上下文**：支持上下文变量的模型（MOIRAI 2.0、Chronos-2、TimeGPT）。
- **多序列**：支持多序列的模型（MOIRAI 2.0、Chronos-2、TimeGPT）。
- **高级云端质量**：TimeGPT，前提是接受数据外发。

高级模型无法弥补糟糕的数据结构。在更换模型之前，先检查数据集的质量、频率、预测期长度和上下文变量。

## 安装本地模型

本地模型必须通过模型管理器（设置 → Forecast）或 Forecast 工作区的模型标签页来安装。它们按家族不同从 Hugging Face 或 GitHub 下载，然后存储在本地的 `~/.local/share/cl-go-dash/forecast-models/` 目录下。
