# Systems Architecture Area Guide

- For `domains/systems-architecture/*` skill rewrites, mine primary sources for operational numbers, endpoint-placement rules, and failure economics before drafting; generic "good abstractions, clear interfaces" language reads well but scores poorly because it does not change agent behavior.
- In architecture skills, prefer boundary decisions over slogans: spell out when to centralize vs distribute, when retries become a throughput bug, when repair matters more than redundancy, and what hidden cost makes the wrong path seductive in production.
- Keep the skill body focused on decision trees, anti-patterns, and fallback playbooks, and push longer source maps or historical context into adjacent reference files so the loaded skill stays dense and actionable.
