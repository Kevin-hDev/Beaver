# Forecast ツール

7つの Forecast ツールは制御された流れを構成します。大きな結果は Forecast ストレージに残し、LLM は小さな識別子を交換します。

## 推奨順序

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

その後、ノート、シナリオ、アンサンブルに `forecast_analyze` を使ってください。

## `forecast_data_audit`

各データセットの最初の予測前に呼び出してください。データまたはファイル、対象、日付、頻度、期間、正確な信頼水準を渡してください。

日付、重複、欠損期間、無効値、履歴、系列、将来、外れ値を検証し、有効なら `data_profile_id` を返します。

## `forecast_models`

現在の方針と区間能力を確認してください。手動では固定モデルを確認し、Auto では `data_profile_id` を渡して候補を1つ選び、`selection_id` を保持してください。

ハードウェア情報はこの回答だけに含まれます。信頼水準を丸めないでください。

## `forecast`

プロファイル、対象、日付、期間、頻度、変更しない信頼水準で実行してください。系列と共変量は対応時だけ追加してください。

Auto ではモデル、`selection_id`、選択元、許可理由も渡します。回答は `analysis_id` を返します。

## `forecast_read`

`analysis_id` を省略すると分析一覧、指定すると1件を読みます。`offset` と `limit` を使い、1ページ最大200点です。

分解、残差異常、時系列順の置換重要度、ドリフトを返す場合があります。利用不能な場合に代替値を作らないでください。

## `forecast_backtest`

保存済み分析で制限付きローリング検証を実行してください。モデルと Naive、Seasonal Naive、Drift、ETS を同じ期間で評価します。

状態と失敗を必ず確認してください。

## `forecast_compare_models`

誤差、カバレッジ、時間、観測メモリ、ベースライン状態を含む保存済み順位を読んでください。完全な比較結果がある場合だけ最良と呼んでください。

## `forecast_analyze`

`annotate`、`scenario`、`scenario_update`、`scenario_delete`、`ensemble` を使ってください。アンサンブルは複数モデルのバックテスト成功後だけ作成し、MASE の逆数による重み付けと独立評価未実施を説明してください。

## 流れの再開始

データ、マッピング、対象、頻度、期間、信頼水準、共変量、系列構造、資源が変わったら、`forecast_data_audit` と `forecast_models` をやり直してください。
