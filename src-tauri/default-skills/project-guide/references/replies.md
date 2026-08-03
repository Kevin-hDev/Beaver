# Replies

Interpret only replies shown on the current screen.

| Reply | Effect |
| --- | --- |
| Displayed letter or number | Select that action or open that displayed category |
| `OK` | Walk pending ranked actions until interaction, instruction-only work, a gap, blocker, or failure |
| `details` | Show exact capability identifiers, evidence, execution behavior, and lookahead without mutation |
| `back` | Render the prior screen without rescanning |
| `recap` | Summarize the existing guide conversation when one exists |
| `explain <key>` | Explain one displayed action in two or three plain lines |
| `explain project` | Summarize verified project context when durable context exists |
| `skip` | Record the displayed action as intentionally left for this session |
| `stop` | Return one closing line and end the loop |

## Execution behavior

- **Complete now:** Run the selected capability to its normal completion, then rescan.
- **Interactive handoff:** Start the selected capability, let it collect its own decisions, and resume the guide only after it returns.
- **Instruction only:** Show what the user must do and invoke nothing.

Determine each behavior only from the selected capability's real contract and the action requested.
