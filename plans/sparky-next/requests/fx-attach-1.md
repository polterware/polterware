# fx-attach-1: eager attachment recovery at core startup

Owner: core wiring integrator. No contract change.

Attachment service initialization now sweeps orphan files and missing-file rows before publishing the service or accepting any imports. Initialization is serialized, so parallel first requests cannot sweep another request's import. Recovery metadata deletes use the SQLite domain path and append system activity.

`OpenRuntime` still initializes attachment services lazily. Its `wiring.go` is outside fx-attach ownership. To meet the literal startup timing, apply the following change in `sparky/backend/compositions/core/wiring.go` after opening SQLite, before `WireSearch`:

```go
attachmentSvc := attachmentMeta{store: store.Attachments()}
if _, err := attachmentSvc.service(); err != nil {
    _ = store.Close()
    return Runtime{}, fmt.Errorf("initialize attachments: %w", err)
}
```

Replace `Attachments: attachmentMeta{store: store.Attachments()},` with `Attachments: attachmentSvc,` in the services literal.

Verification: `TestCoreAttachmentAuditAndRecoveryBeforeImport` covers recovery before import, metadata audit, and no symlink traversal. Extend the restart assertion to inspect the orphan directory immediately after `OpenRuntime` and before the first attachment operation when applying this request.
