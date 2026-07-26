# Overview

Forecast is directly linked to the active conversation. The LLM prepares or researches data, runs calculations and explains results. The chat remains the command center, while two complementary surfaces display and explore each analysis.

## Main workflow

The normal workflow is:

1. the user describes what to predict in the chat;
2. the LLM reads, creates or enriches the required data;
3. Forecast audits data quality;
4. Manual mode enforces the user's model, while Auto chooses from safe candidates;
5. Forecast computes and saves the prediction;
6. the panel immediately displays the main result;
7. the user continues the conversation or opens the Forecast workspace.

There is no separate Forecast chat. Ask for an explanation, comparison or rerun by writing a normal message.

## Complementary surfaces

| Surface | Purpose |
| --- | --- |
| Chat | Prepare data, guide the LLM and request explanations |
| Forecast panel | Quickly read the chart, key indicators and warnings |
| Forecast workspace | Explore data, charts, evaluations, scenarios, notes and the report |

The panel intentionally stays compact. The workspace opens in a dedicated window without hiding or replacing the conversation.

## Forecast workspace

The workspace remains linked to the active session and analysis. Selecting another analysis in the panel updates the open window automatically.

Its sections are:

| Section | Content |
| --- | --- |
| Data | Dataset summary, mapping, quality and row preview |
| Forecast | Main chart, uncertainty, seasonality, filters and prediction table |
| Evaluation | Temporal backtest, baselines and interval reliability |
| Comparison | Comparable model ranking and optional ensemble creation |
| Scenarios | Create and edit assumptions |
| Notes | Context, risks, decisions and annotations |
| Report | Detailed analysis and exports |

## Saved analysis

An analysis keeps the effective columns and settings, data-quality profile, model and selection source, central prediction and intervals, scenarios, notes, backtests and the provenance needed for reproducibility.

## Key point

Forecast produces a structured estimate, not certainty. Read every curve together with data quality, uncertainty, baselines and the limits of the available context.
