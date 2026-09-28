import { PageContent, PageEmpty, PageToolbar } from "@polterware/frontend-design-system";

/** Inbox — items that need a person. Nothing feeds it yet, so it states that honestly. */
export function InboxPage() {
  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Leading>
          <span class="pw-page-toolbar-title">Inbox</span>
        </PageToolbar.Leading>
      </PageToolbar.Root>
      <PageContent class="p-0">
        <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
          <PageEmpty
            title="Nothing needs you right now."
            description="Open questions, drift findings, and facts to verify will collect here."
          />
        </div>
      </PageContent>
    </div>
  );
}
