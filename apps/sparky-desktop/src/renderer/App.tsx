import { PageHeader } from "@polterware/frontend-design-system";
import { features } from "./features";

export function App() {
  return (
    <main class="flex h-full flex-col items-center justify-center gap-4 p-8">
      <PageHeader
        title="Sparky"
        subtitle="Scaffold stub — core and features ship in Wave 1/2."
      />
      <p class="text-sm text-[var(--color-text-secondary)]">
        Registered features: {features.map((f) => f.id).join(", ")}
      </p>
    </main>
  );
}
