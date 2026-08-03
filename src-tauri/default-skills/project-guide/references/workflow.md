# Delivery Workflow

Use this sequence as a state-aware curriculum, not as a mandatory chain for every request.

| Stage | Outcome | Skip only when |
| --- | --- | --- |
| Clarify | The idea and constraints are precise | The request is already unambiguous |
| Specify | Observable scope and completion conditions exist | The change is truly small and the user provided a complete contract |
| Plan | Phases, risks, files, and checks are explicit | The change is a single obvious edit with no material design choice |
| Implement | The requested behavior exists phase by phase | Never skip for implementation work |
| Validate and test | Required behavior and gates have evidence | Never skip a required gate |
| Review | An independent verdict covers the full change | Never skip when the requested workflow requires review |
| Commit | The accepted change has an atomic record | The user did not request version-control delivery |
| Review request | The current branch is ready for merge review | The user did not request external delivery |

When an end-to-end orchestrator is available and the user explicitly wants the complete flow, you may offer it as one idle choice. Otherwise guide one verified stage at a time.
