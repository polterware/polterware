# Security smoke (004 implement)

| Check | Result |
| --- | --- |
| Credential secrets re-displayed after create | Pass — write-only field cleared; list has no secret field |
| Renderer unrestricted Node/OS | Pass — features use props/bridge only |
| High-risk run command | Pass — dialog CTA is “Request execution”; toast notes approval may apply |
| Self-approve UI | Pass — approval detail always shows Approve/Reject for authorized path; fixture mode does not silent-apply without UI action |
| Process stop | UI only; no silent core apply without adapter |
| MCP bypass | Not introduced |

Date: 2026-08-03
