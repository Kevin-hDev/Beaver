# LLM agents

LLM agents can use Forecast as a specialized engine. Their role is not limited to reading a file and clicking a tool: they can prepare data, search for context on the web, build a dataset, run the prediction, and explain the result.

## What the agent can do

An agent can intervene at several moments:

| Step | Agent role |
| --- | --- |
| Preparation | Read an Excel, CSV, or JSON file |
| Research | Fetch external information from the web |
| Dataset | Create or complete useful columns |
| Run | Call `forecast` with the right parameters |
| Read | Use `forecast_read` to retrieve the result |
| Scenario | Create hypotheses and rerun the model |
| Explanation | Summarize trend, uncertainty, variables, and limits |

Example: for a financial forecast, the agent can read the local file, search for recent market context, produce columns like `news_score` or `event_flag`, then run Forecast.

## Recommended workflow

The agent must follow this order:

1. understand the user's request;
2. inspect the available data;
3. identify the target to predict;
4. identify the dates, the frequency, and the horizon;
5. search for or create the useful contextual variables if needed;
6. check that the future rows are consistent;
7. choose a compatible model;
8. run `forecast`;
9. review the result with `forecast_read`;
10. explain the forecast and propose useful scenarios.

## Data creation by the agent

The agent can create data if the user asks for it or if the prediction requires it.

Examples:

- add a `weekend` column from the date;
- create `month_end_flag`;
- turn a web event into a numeric score;
- fill a future zone with weather hypotheses;
- build a test dataset to validate a workflow;
- convert an Excel file into usable JSON.

The agent must always explain which columns it created and why.

## Safety and quality rules

The agent must not silently invent important data.

If it creates a variable, it must distinguish:

- data read from a file;
- data found on the web;
- computed data;
- simulation hypothesis.

This separation is essential so the user knows what is real, computed, or assumed.

## Slash commands

Slash commands act as quick guides for agents and users.

Examples:

- `/forecast`: understand the Forecast module;
- `/forecast-predict`: prepare and run a prediction;
- `/forecast-dataset`: build a clean dataset;
- `/forecast-scenarios`: create useful hypotheses;
- `/forecast-cmd`: understand the available tools.

These commands must give a short, clear, and directly actionable procedure.
