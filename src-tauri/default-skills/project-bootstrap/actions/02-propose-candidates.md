# 02 - Propose Candidates

Build two or three materially different technical candidates and support unstable claims with current official evidence.

## Input

- Use the complete approved needs checklist and recorded technology preferences.
- Use current official documentation, support, licensing, and pricing sources.

## Output

- Return two or three distinct candidates in a comparison table with architecture, interface, runtime, data, security, integration, deployment or distribution, cost, performance fit, evidence, and risks.

## Process

1. **Read heuristics.** Apply [decision-heuristics.md](../references/decision-heuristics.md) as fallible selection guidance, not as authority.
2. **Derive families.** Map the project type and confirmed constraints to plausible architecture, interface, runtime, data, access, deployment, and distribution families.
3. **Create a real spread.** Produce two or three candidates that differ materially on at least one high-impact dimension. Reject cosmetic variants that merely swap a minor library or vendor plan.
4. **Preserve the service path.** For a hosted multi-user product, include explicit front end, back end, database, authentication, hosting, tenancy, and operating-cost choices. For other project types, map those concerns to the applicable client, core runtime, persistence, access, packaging, distribution, and support choices.
5. **Verify unstable claims.** Open current official primary sources before asserting versions, compatibility, maintenance status, license terms, platform support, service limits, or prices. Collect at most 12 sources per candidate per batch, keep a source cursor, and continue until every material unstable claim is supported or marked unavailable.
6. **Estimate cost.** Estimate monthly operating cost at the six-month scale and add one-time or distribution costs when relevant. State date, region, usage assumptions, excluded taxes, and uncertainty. Never present an unavailable price as zero.
7. **Expose risks.** Give each candidate one to three concrete risks covering lock-in, operational load, learning curve, ecosystem, performance, scale, distribution, or security. Give no candidate a risk-free label.
8. **Challenge preferences.** When a preferred technology conflicts with confirmed needs, show the conflict and exclude it or preserve it only as an explicitly risky candidate for audit.
9. **Render without choosing.** Show one table with non-empty candidate fields and nearby evidence links. Do not rank or select a winner before independent audit.

## Stop conditions

- Stop and return to needs when constraints cannot support two genuinely different plausible candidates.
- Stop a candidate from advancing when a material compatibility, support, license, or price claim requires current evidence that remains unavailable.
- Do not install tools, create accounts, provision services, write documentation, or scaffold files.

## Test

- Confirm that the table contains two or three candidates and that at least two differ materially in architecture, runtime, data, deployment or distribution, or operational ownership.
- Confirm that every candidate covers the project-type equivalents of front end, back end, database, authentication, hosting, cost, and risks.
- Confirm that every unstable material claim has a current official source, date, and applicable assumptions.
