# W1-DOMAIN request 1: repository fakes outside Owns

- Reason: W1-DOMAIN says "in-memory fakes of every repository port", but Owns permits only memory, mind, tag, focus, review, activity and settings. The frozen skeletons in the remaining domains still panic. Ownership takes precedence; this sub-task is stopped.
- Requested owner action: assign the remaining fake implementations to their domain owners, or explicitly extend ownership in a follow-up. No contract changes are needed.
- Exact requested paths: `sparky/backend/domains/{attachments,place,maintenance,search,reminder,schedule,clock,ids,tx}/fake/**`.
- Required behavior: replace each `panic("...: not implemented")` with an implementation of its existing frozen port. Repositories must accept context, copy mutable values, and share transaction state where related rows require it. Clock and IDs should be injectable deterministic test values; the ID generator must return ULIDs.
- Available integration: `activity/fake.NewStore()` implements `tx.Manager`. Pass the same store to `memory/fake.New`, `mind/fake.New`, `tag/fake.New`, `focus/fake.New`, `review/fake.New`, `activity/fake.New`, and `settings/fake.New`. `Store.Put/Remove` and generic `Read/All/Map` participate in transactions, including rollback and cancellation. These are testing helpers, not frozen wire contracts.
- Shared suite: `tests/repotest/**` is also outside W1-DOMAIN Owns. W1-STORE should extract the owned repository contract cases into that shared suite and run them against SQLite; W1-DOMAIN commits contract cases beside each owned fake.
- No edits were made to the requested paths or frozen ports.
