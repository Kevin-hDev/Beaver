# Covariates

Covariates are the contextual variables that can influence the target. They describe the environment around the data to predict.

## Definition

A covariate answers the question: "what can help the model understand why the target rises, falls, or changes rhythm?"

Examples:

| Domain | Target | Possible covariates |
| --- | --- | --- |
| Restaurant | Orders | weather, public holiday, local event, promo |
| Finance | Price or volume | VIX, rates, news score, Bitcoin, dollar index |
| SaaS | Server load | active users, campaign, incident, release |
| Retail | Sales | stock, discount, season, store traffic |

## Why they matter

Without a covariate, the model mostly looks at the past of the target.

With covariates, the model can also use the context:

- "sales often rise when a promo is active";
- "orders drop when it rains heavily";
- "the price reacts when the VIX rises";
- "load increases after a marketing campaign".

They therefore make it possible to produce a more contextualized forecast than a simple extension of the past.

## Types of contextual variables

A covariate can be:

| Type | Example | Reading |
| --- | --- | --- |
| Numeric | `temperature = 28` | Measured value |
| Percentage | `discount_pct = 15` | Intensity of an effect |
| Binary | `weekend = 1` | Yes / no |
| Event | `concert_local = 1` | Event present |
| Score | `news_score = 0.72` | Computed indicator |
| Calendar | `jour_ferie = 1` | Temporal context |

## History and known future

A covariate is more useful if it exists in two zones:

- history: the model learns how it influenced the target;
- known future: the model uses its future values to predict the target.

Example:

```text
date        commandes   pluie_mm   weekend
2026-05-01  120        0          0
2026-05-02  148        4          1
2026-05-03             12         1
```

The future target is empty, but `pluie_mm` and `weekend` are known. The model can therefore forecast while accounting for these future conditions.

## Variables in scenarios

In a contextual scenario, the user modifies the future covariates.

Examples:

- increase `vix_close` by 20%;
- set `promo_active` to 1 for a week;
- lower `temperature` by 5 degrees;
- simulate a high `breach_alert_level`;
- change `trafic_indice` on future days.

Forecast then reruns the model with this new context to produce a new trajectory.

## Finance variable dictionary

In a finance dataset, contextual variables may have technical names. This table explains the variables visible in finance scenarios.

| Variable | What it represents | Simple reading |
| --- | --- | --- |
| `nasdaq_return_pct` | Nasdaq variation in percentage | Measures whether the tech market rises or falls |
| `vix_close` | VIX level at close | Measures market fear or volatility |
| `btc_close_usd` | Bitcoin price in dollars | Acts as a risk signal or speculative appetite |
| `usd_index_dxy` | US dollar index | Measures the strength of the dollar against other currencies |
| `treasury_10y_pct` | US 10-year rate | Represents the long-term cost of money |
| `sector_etf_volume_musd` | Volume traded on a sector ETF, in millions of dollars | Measures activity in a specific sector |
| `breach_alert_level` | Alert level tied to a flaw or cyber incident | Represents stress specific to the cybersecurity sector |
| `zero_day_news_score` | News score around zero-day flaws | Measures the intensity of critical cyber news |
| `gov_contract_flow_index` | Public contract flow index | Represents the momentum of government contracts |
| `earnings_heat_index` | Tension index around financial earnings | Measures the importance or sensitivity of the earnings period |
| `ai_capex_signal` | Investment signal tied to AI spending | Represents the strength of the AI investment theme |
| `fed_event_flag` | US central bank event indicator | Often equals 1 when a Fed event is present |
| `option_expiry_flag` | Options expiration indicator | Signals a day where options may influence moves |
| `month_end_flag` | End-of-month indicator | Signals rebalancing or monthly close effects |
| `weekend` | Weekend indicator | Mostly used for daily data with a calendar effect |

## How to read these variables

These variables do not predict anything on their own. They give context to the model.

Examples:

- if `vix_close` rises, the market is often more nervous;
- if `fed_event_flag` equals 1, the day may be more sensitive to rate announcements;
- if `zero_day_news_score` rises, cyber stocks may react;
- if `sector_etf_volume_musd` rises, sector activity is stronger;
- if `month_end_flag` equals 1, some moves may come from monthly closes.

The model learns in the history whether these variables have already accompanied moves of the target.

## Use in a scenario

In the Scenarios tab, modifying these variables amounts to stating a hypothesis.

Examples:

| Hypothesis | Possible modification |
| --- | --- |
| More stressed market | `vix_close` +20% |
| Stronger dollar | `usd_index_dxy` +2% |
| Fed day | `fed_event_flag` = 1 |
| Heavy cyber news | `zero_day_news_score` +30% |
| Sensitive month-end | `month_end_flag` = 1 |

After rerun, Forecast recomputes the trajectory with this new future context.

## Pitfalls to avoid

A covariate can degrade the result if it is poorly prepared.

Things to avoid:

- variable empty on the known future;
- constant variable that provides no information;
- free text not converted into a usable value;
- variable that indirectly contains the future target;
- mixed units within the same column;
- invented future value without a clear hypothesis.
