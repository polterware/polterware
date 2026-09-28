/**
 * Local-first privacy note for Settings → General.
 * Trust copy only — not a terms-of-service document.
 */
export function PrivacyCard() {
  return (
    <div class="space-y-2">
      <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
        Privacy
      </p>
      <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
        Polterware runs as a local Desktop on your machine. There is no account
        sign-in and no hosted workspace for day-to-day use.
      </p>
      <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
        Project data stays on this device. Provider credentials used through
        Integrations remain in the OS or CLI credential store — Polterware does
        not store provider tokens in the app database.
      </p>
    </div>
  );
}
