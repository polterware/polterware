import { Button, PageEmpty } from "@polterware/frontend-design-system";

export function IntegrationsConnectedEmpty(props: { onOpenCatalog?: () => void }) {
  return (
    <PageEmpty
      title="No connected integrations."
      description="Open the catalog to connect a CLI provider."
      action={
        props.onOpenCatalog ? (
          <Button size="sm" variant="primary" onClick={() => props.onOpenCatalog?.()}>
            Open catalog
          </Button>
        ) : undefined
      }
    />
  );
}

export function IntegrationsCatalogFilterEmpty() {
  return <PageEmpty title="No providers match" description="Try another name or category." />;
}
