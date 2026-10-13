import { createEffect, createMemo, createResource, createSignal, ErrorBoundary, For, onCleanup, onMount, Show, type Component } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  Button, CalendarClock, Clock, DetailPanel, DesignSystemProvider,
  Dialog, DitherIconTile, Folder, HelpCircle, Inbox, MapPin, MenuItem, MenuList, PageEmpty, PageError,
  FloatingWindow, PageHeader, PageLoading, PageOffline, Plus, Popover, Search, Settings, StatusChip, ToastRegion, TreeView, pushToast, type TreeViewNode,
} from "@polterware/frontend-design-system";
import { MainFrame, MainSurfaceFrame, ShellRoot, SidebarFrame } from "@polterware/frontend-shell";
import type { MessageShape } from "@bufbuild/protobuf";
import type { MindSchema } from "@polterware/sparky-contracts";
import type { ShellBridge } from "./bridge";
import type { SparkyFeature } from "./feature";
import { ShellContext, type MemoryDetailRenderer } from "./context";
import { Commander } from "./commander";
import { MemoryComposer } from "./memory-composer";
import { createCoreConnection, independentQueryKey, reminderLabel } from "./connection";
import { DEFAULT_PATH, deepLinkPath, normalizePath, pushRoute, resolveRoute, SHELL_ROUTES, sidebarItems, type RouteHistory } from "./route";
import { isCollectionTarget, isEditing, memoryShortcut, shellShortcut, type MemoryShortcut } from "./shortcuts";
import "./shell.css";

const icons: Record<string, Component<{ size?: number }>> = { CalendarClock, Clock, Folder, Inbox, MapPin, Search, Settings };
const titlebarIconClass = "control-field control-field-sm inline-flex w-[var(--control-height-sm)] shrink-0 cursor-pointer items-center justify-center !rounded-full !p-0 text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text-primary)] focus-visible:outline-none";
export function mindTree(minds: MessageShape<typeof MindSchema>[], parentId = "", ancestors = new Set<string>()): TreeViewNode[] {
  return minds.filter(mind => (mind.parentId ?? "") === parentId && mind.id && !ancestors.has(mind.id)).map(mind => ({
    id: mind.id!, label: mind.name ?? "Untitled Mind", children: mindTree(minds, mind.id!, new Set([...ancestors, mind.id!])),
  }));
}

export function SparkyShell(props: { features: readonly SparkyFeature[]; bridge?: ShellBridge; onReady?: () => void }) {
  const core = createCoreConnection(props.bridge);
  createEffect(() => { if (core.state() !== "loading") props.onReady?.(); });
  const [history, setHistory] = createSignal<RouteHistory>({ entries: [DEFAULT_PATH], index: 0 });
  const path = () => history().entries[history().index] ?? DEFAULT_PATH;
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  const [settingsPanel, setSettingsPanel] = createSignal("");
  const [palette, setPalette] = createSignal<"commands" | "quick-add" | "search" | null>(null);
  const [composerOpen, setComposerOpen] = createSignal(false);
  const [composerExpanded, setComposerExpanded] = createSignal(false);
  const [selectedMemoryId, setSelectedMemoryId] = createSignal<string | null>(null);
  const [expanded, setExpanded] = createSignal(false);
  const [detailRenderer, setDetailRenderer] = createSignal<MemoryDetailRenderer>();
  const [onboardingOpen, setOnboardingOpen] = createSignal((() => {
    try { return localStorage.getItem("sparky.shell.onboarding") !== "complete"; }
    catch { return true; }
  })());
  const [onboardingStep, setOnboardingStep] = createSignal(0);
  const [authorizationBusy, setAuthorizationBusy] = createSignal(false);
  const [authorizationError, setAuthorizationError] = createSignal<string | null>(null);
  const [deepLink, setDeepLink] = createSignal<URL | null>(null);
  let collectionShortcuts: ((action: MemoryShortcut) => void) | undefined;
  const [helpMenuOpen, setHelpMenuOpen] = createSignal(false);
  let helpAnchor: HTMLButtonElement | undefined;
  const [helpOpen, setHelpOpen] = createSignal(false);
  const [dataRevision, setDataRevision] = createSignal(0);
  const [expandedMinds, setExpandedMinds] = createSignal<string[]>([]);
  let main: HTMLElement | undefined;
  let restoredFocus: HTMLElement | undefined;
  const route = createMemo(() => resolveRoute(props.features, path()));
  const hasFeatureRoute = createMemo(() => props.features.some(feature => feature.id !== "shell" && feature.routes.some(item => item === route()?.route)));
  const items = createMemo(() => sidebarItems(props.features));
  const settingsPanels = createMemo(() => props.features.flatMap(feature => feature.settingsPanels ?? []));
  const settingsGroups = createMemo(() => [...new Set(settingsPanels().map(panel => panel.group))]);
  const activeSettings = createMemo(() => settingsPanels().find(panel => panel.id === settingsPanel()) ?? settingsPanels()[0]);
  const detailSections = createMemo(() => props.features.flatMap(feature => feature.detailSections ?? []).sort((a, b) => a.order - b.order));
  const memoryDetail = createMemo(() => props.features.find(feature => feature.memoryDetail)?.memoryDetail);
  const memoryCollection = createMemo(() => props.features.find(feature => feature.memoryCollection)?.memoryCollection);
  const sidebarTree = createMemo(() => props.features.find(feature => feature.sidebarTree)?.sidebarTree);
  const canRequestNotifications = () => typeof props.bridge?.app.requestNotificationAuthorization === "function";
  const notificationGranted = () => ["authorized", "provisional", "ephemeral"].includes(core.status()?.permission ?? "");
  const queryKey = () => independentQueryKey(core.state(), dataRevision(), props.bridge);
  const [minds] = createResource(() => sidebarTree() ? false : queryKey(), async () => (await props.bridge!.minds.list({})).minds ?? []);
  const [inboxCount] = createResource(queryKey, async () => (await props.bridge!.memories.list({ inboxOnly: true, status: "active", limit: 1 })).total);
  const tree = createMemo(() => mindTree(minds.error ? [] : minds() ?? []));
  const params = createMemo(() => route()?.params ?? {});
  const searchParams = createMemo(() => new URLSearchParams(path().split("?")[1] ?? ""));
  const [memory] = createResource(() => core.state() === "live" && selectedMemoryId() ? `${selectedMemoryId()}:${dataRevision()}` : false,
    async () => (await props.bridge!.memories.get({ id: selectedMemoryId()!, include: { note: true } })).memory);
  const detailRoute = createMemo(() => selectedMemoryId() ? resolveRoute(props.features, `/memory/${selectedMemoryId()}`) : undefined);
  function closeDetail() {
    setSelectedMemoryId(null); setExpanded(false);
    if (restoredFocus?.isConnected) restoredFocus.focus();
  }
  function openMemory(id: string) {
    restoredFocus = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    setSelectedMemoryId(id);
  }
  function navigate(raw: string) {
    const normalized = normalizePath(raw);
    if (normalized === "/settings") { setSettingsOpen(true); return; }
    if (/^\/memory\/[^/]+$/.test(normalized)) { openMemory(decodeURIComponent(normalized.slice(8))); return; }
    closeDetail();
    const next = resolveRoute(props.features, normalized) || SHELL_ROUTES.some(route => route.path === normalized) || /^\/mind\/[^/]+$/.test(normalized) ? raw : DEFAULT_PATH;
    // Keep query parameters for feature search/calendar state.
    setHistory(current => pushRoute(current, next));
    queueMicrotask(() => main?.focus());
  }
  function moveHistory(delta: number) {
    const index = history().index + delta;
    if (index < 0 || index >= history().entries.length) return;
    closeDetail(); setHistory(current => ({ ...current, index }));
    queueMicrotask(() => main?.focus());
  }
  function finishOnboarding() {
    try { localStorage.setItem("sparky.shell.onboarding", "complete"); } catch { /* Session-only when storage is unavailable. */ }
    setOnboardingOpen(false);
  }
  async function enableNotifications() {
    if (!canRequestNotifications() || authorizationBusy()) return;
    setAuthorizationBusy(true);
    setAuthorizationError(null);
    try {
      await props.bridge!.app.requestNotificationAuthorization();
      await core.refresh();
    } catch (error) {
      setAuthorizationError(error instanceof Error ? error.message : "Notification authorization failed.");
    } finally { setAuthorizationBusy(false); }
  }
  const commands = createMemo(() => [
    ...SHELL_ROUTES.map(item => ({ id: `navigate:${item.path}`, label: `Go to ${item.title}`, keywords: ["navigate"], run: () => navigate(item.path) })),
    ...props.features.flatMap(feature => feature.routes).filter(item => !SHELL_ROUTES.some(defaultRoute => defaultRoute.path === item.path) && !item.path.includes(":"))
      .map(item => ({ id: `navigate:${item.path}`, label: `Go to ${item.title}`, run: () => navigate(item.path) })),
    { id: "quick-add", label: "New memory", keywords: ["add", "create"], run: () => { setPalette(null); setComposerOpen(true); } },
    { id: "settings", label: "Open Settings", run: () => setSettingsOpen(true) },
    ...props.features.flatMap(feature => feature.commands ?? []),
  ]);
  const commandSources = createMemo(() => props.features.flatMap(feature => feature.commandSources ?? []));
  const context = {
    features: props.features, path, params, searchParams, navigate, selectedMemoryId, openMemory, closeDetail, detailSections, settingsPanels, deepLink,
    registerCollectionShortcuts: (handler: (action: MemoryShortcut) => void) => {
      collectionShortcuts = handler;
      return () => { if (collectionShortcuts === handler) collectionShortcuts = undefined; };
    },
    registerMemoryDetail: (component: MemoryDetailRenderer) => {
      setDetailRenderer(() => component);
      return () => { if (detailRenderer() === component) setDetailRenderer(undefined); };
    },
  };
  async function routeLink(link: { url: string; route: string }) {
    let url: URL;
    try { url = new URL(link.url); } catch { return; }
    if (url.protocol !== "sparky:") return;
    setDeepLink(url);
    let handled = false;
    for (const feature of props.features) {
      if (feature.onDeepLink?.(url)) { handled = true; break; }
    }
    if (url.hostname === "memory" && url.searchParams.get("action") === "focus") {
      const id = url.pathname.replace(/^\//, "");
      navigate("/focus");
      if (id) openMemory(id);
      return;
    }
    if (handled) return;
    const target = deepLinkPath(link.url);
    if (target) navigate(`${target}${url.search}`);
  }
  onMount(() => {
    try {
      const saved: unknown = JSON.parse(localStorage.getItem("sparky.shell.minds.expanded") ?? "[]");
      if (Array.isArray(saved) && saved.every(value => typeof value === "string")) setExpandedMinds(saved);
    } catch { setOnboardingOpen(true); }
    void core.refresh();
    const timer = setInterval(() => void core.refresh(), 5000);
    const offChange = props.bridge?.onChange(() => { setDataRevision(value => value + 1); void core.refresh(); });
    const offLink = props.bridge?.app.onDeepLink(link => { void routeLink(link); });
    void props.bridge?.app.getDeepLink().then(link => { if (link) void routeLink(link); }).catch(() => {});
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented) return;
      const shortcut = shellShortcut(event, isEditing(event.target));
      if (settingsOpen() || onboardingOpen() || helpOpen()) return;
      if (!shortcut) {
        if (palette() !== null) return;
        const action = memoryShortcut(event, isEditing(event.target));
        if (action && collectionShortcuts && isCollectionTarget(event.target)) { event.preventDefault(); collectionShortcuts(action); }
        return;
      }
      if (palette() !== null && !["commander", "quick-add"].includes(shortcut)) return;
      event.preventDefault();
      switch (shortcut) {
        case "commander": setPalette(current => current === "commands" ? null : "commands"); break;
        case "quick-add": setPalette(null); setComposerOpen(true); break;
        case "settings": setSettingsOpen(true); break;
        case "back": moveHistory(-1); break;
        case "forward": moveHistory(1); break;
        case "close-detail": closeDetail(); break;
        case "search": {
          const search = main?.querySelector<HTMLInputElement>('input[type="search"], input[data-list-search]');
          if (search) search.focus(); else setPalette("search");
          break;
        }
        default: {
          const target = ["/inbox", "/today", "/calendar", "/map", "/focus"][Number(shortcut.slice(-1)) - 1];
          if (target) navigate(target);
        }
      }
    };
    document.addEventListener("keydown", onKey);
    onCleanup(() => { clearInterval(timer); offChange?.(); offLink?.(); document.removeEventListener("keydown", onKey); });
  });
  createEffect(() => { if (core.state() === "offline") closeDetail(); });
  let reminderToast = "";
  createEffect(() => {
    if (onboardingOpen() || core.state() !== "live") return;
    const message = reminderLabel(core.status());
    const problem = message === "Reminders unavailable" || message.startsWith("Reminders: permission") || message === "Reminders: refresh pending";
    if (!problem) { reminderToast = ""; return; }
    if (message === reminderToast) return;
    reminderToast = message;
    pushToast(message);
  });
  const title = () => route()?.route.title ?? SHELL_ROUTES.find(item => item.path === normalizePath(path()))?.title ?? "Mind";
  const boundary = (error: Error, reset: () => void) => <PageError title="This feature failed" message={error.message} onRetry={reset} />;
  return <ShellContext.Provider value={context}>
    <DesignSystemProvider storagePrefix="sparky" expandDetail={() => { if (selectedMemoryId()) setExpanded(true); }}
      matchesShortcut={(id, event) => id === "shell.toggle_detail" && shellShortcut(event, false) === "close-detail"}
      createShortcutLabel={id => () => id === "shell.toggle_detail" ? "⌘J" : ""}>
      <ShellRoot glass={false} class="sparky-shell">
        <FloatingWindow.Root open={composerOpen()} minimized expanded={composerExpanded()} controls="close"
          aria-label="New memory" storageKey="sparky.memory.composer" bubble={<Plus size={16} />}
          onOpenChange={next => { setComposerOpen(next); if (!next) setComposerExpanded(false); }}
          onMinimizedChange={() => {}} onExpandedChange={setComposerExpanded}>
        <FloatingWindow.Workspace>
        <div class="sparky-workspace">
          <SidebarFrame class="sparky-sidebar" aria-label="Sidebar">
            <MenuList>
              <MenuItem aria-label="Open Commander" onClick={() => setPalette("commands")}><Search size={15} /><span class="flex-1">Search</span></MenuItem>
            </MenuList>
            <nav aria-label="Main navigation">
              <MenuList><For each={items()}>{item => <MenuItem aria-label={item.label} aria-current={normalizePath(path()) === normalizePath(item.path) ? "page" : undefined}
                selected={normalizePath(path()) === normalizePath(item.path)} onClick={() => navigate(item.path)}>
                <Dynamic component={icons[item.icon ?? ""] ?? Folder} size={15} /><span class="flex-1">{item.label}</span>
                <Show when={item.path === "/inbox" && !inboxCount.error && inboxCount() !== undefined}><span aria-label="Active Inbox memories">{inboxCount()}</span></Show>
                <Show when={item.badge !== undefined && item.path !== "/inbox"}><span>{item.badge}</span></Show>
              </MenuItem>}</For></MenuList>
            </nav>
            <Show when={sidebarTree()} fallback={
              <Show when={core.state() === "live"}>
                <Show when={!minds.error} fallback={<p role="status">Minds unavailable</p>}>
                  <Show when={tree().length}><TreeView label="Minds" nodes={tree()} expandedValue={expandedMinds()} selectedValue={params().id ? [params().id!] : []}
                    onExpandedChange={details => { setExpandedMinds(details.expandedValue); try { localStorage.setItem("sparky.shell.minds.expanded", JSON.stringify(details.expandedValue)); } catch { /* Session-only. */ } }}
                    onSelectionChange={details => { const id = details.selectedValue[0]; if (id) navigate(`/mind/${encodeURIComponent(id)}`); }} /></Show>
                </Show>
              </Show>
            }>{treeDef => <Dynamic component={treeDef().component} />}</Show>
            <div class="sparky-sidebar-footer">
              <Button variant="ghost" class="min-w-0 flex-1 justify-start !rounded-[var(--radius-item)]" onClick={() => setSettingsOpen(true)}><Settings size={15} />Settings</Button>
              <button type="button" ref={helpAnchor} class={titlebarIconClass} aria-label="Help" aria-expanded={helpMenuOpen()} aria-haspopup="dialog" onClick={() => setHelpMenuOpen(open => !open)}><HelpCircle size={14} /></button>
              <Popover open={helpMenuOpen()} onClose={() => setHelpMenuOpen(false)} title="Help" anchorRef={() => helpAnchor} placement="top-end" width={240}>
                <MenuList>
                  <MenuItem onClick={() => { setHelpMenuOpen(false); setHelpOpen(true); }}>Keyboard shortcuts</MenuItem>
                  <MenuItem onClick={() => { setHelpMenuOpen(false); setOnboardingStep(0); setOnboardingOpen(true); }}>Show onboarding</MenuItem>
                </MenuList>
              </Popover>
            </div>
          </SidebarFrame>
          <MainFrame as="main" glass={false} class="sparky-main" ref={main} tabIndex={-1} aria-label={title()}>
            <MainSurfaceFrame as="div">
              <Show when={core.state() !== "loading"} fallback={<PageLoading label="Starting Sparky…" />}>
                <Show when={core.state() === "live"} fallback={<PageOffline title="Core offline" message="Sparky cannot reach the local core. Retry to reconnect." onRetry={() => void core.refresh()} />}>
                  <DetailPanel.Root open={!!selectedMemoryId()} expanded={expanded()} activeEntityId={selectedMemoryId()} onOpenChange={open => { if (!open) closeDetail(); }} onExpandedChange={setExpanded} aria-label="Memory details">
                    <DetailPanel.Collection>
                      <Show keyed when={path()}>{(_path) => <ErrorBoundary fallback={boundary}>
                        <Show when={!hasFeatureRoute() && /^\/mind\/[^/]+$/.test(normalizePath(path())) && memoryCollection()} fallback={
                          <Show when={route()} fallback={<><PageHeader title={title()} subtitle={<StatusChip tone="muted" label="Unavailable" />} /><PageEmpty title={`${title()} is unavailable`} description="This feature has not registered a route yet." /></>}>
                            {resolved => <Dynamic component={resolved().route.component} />}
                          </Show>
                        }>{collection => <Dynamic component={collection()} mindId={decodeURIComponent(normalizePath(path()).split("/")[2] ?? "")} />}</Show>
                      </ErrorBoundary>}</Show>
                    </DetailPanel.Collection>
                    <DetailPanel.Content><DetailPanel.Header /><ErrorBoundary fallback={boundary}>
                      <Show when={selectedMemoryId()}>{id => <Show when={memoryDetail() ?? detailRenderer() ?? detailRoute()?.route.component} fallback={
                        <DetailPanel.Body layout="document"><DetailPanel.Main>
                          <Show when={!memory.loading} fallback={<PageLoading label="Loading memory…" />}>
                            <Show when={!memory.error && memory()} fallback={<PageError message="The selected memory could not be loaded." onRetry={() => setDataRevision(value => value + 1)} />}>
                              <DetailPanel.Title title={memory()?.title ?? "Memory"} />
                              <StatusChip tone="muted" label="Details editor unavailable" />
                              <p class="whitespace-pre-wrap">{memory()?.note}</p>
                              <For each={detailSections().filter(section => !section.visible || section.visible(id()))}>{section => <Dynamic component={section.component} />}</For>
                            </Show>
                          </Show>
                        </DetailPanel.Main></DetailPanel.Body>
                      }>{component => <Dynamic component={component()} memoryId={id()} detailSections={detailSections()} />}</Show>}</Show>
                    </ErrorBoundary></DetailPanel.Content>
                  </DetailPanel.Root>
                </Show>
              </Show>
            </MainSurfaceFrame>
          </MainFrame>
        </div>
        </FloatingWindow.Workspace>
        <FloatingWindow.Content>
          <MemoryComposer open={composerOpen()} bridge={props.bridge} live={core.state() === "live"} onCreated={openMemory} onClose={() => setComposerOpen(false)} />
        </FloatingWindow.Content>
        </FloatingWindow.Root>
      </ShellRoot>
      <ToastRegion />
      <Commander open={palette() !== null} mode={palette() ?? "commands"} bridge={props.bridge} commands={commands()} commandSources={commandSources()} live={core.state() === "live"} onClose={() => setPalette(null)} onMemory={openMemory} />
      <Dialog open={settingsOpen()} bodyClass="flex min-h-0 flex-1 flex-col overflow-hidden px-7 py-4 text-[13px] text-[var(--color-text-secondary)]" title="Settings" onClose={() => setSettingsOpen(false)} size="xl">
        <div class="sparky-settings-layout">
          <nav aria-label="Settings panels"><For each={settingsGroups()}>{group => <><p class="text-xs text-[var(--color-text-muted)]">{group}</p><MenuList>
            <For each={settingsPanels().filter(panel => panel.group === group)}>{panel => <MenuItem selected={panel.id === activeSettings()?.id} aria-current={panel.id === activeSettings()?.id ? "page" : undefined} onClick={() => setSettingsPanel(panel.id)}>{panel.label}</MenuItem>}</For>
          </MenuList></>}</For></nav>
          <section aria-label="Selected settings panel"><Show when={activeSettings()} fallback={<PageEmpty title="Settings unavailable" description="Settings panels have not been registered yet." />}>
            {panel => <Show keyed when={panel().id}>{(_id) => <ErrorBoundary fallback={boundary}><Dynamic component={panel().component} /></ErrorBoundary>}</Show>}
          </Show></section>
        </div>
      </Dialog>
      <Dialog open={onboardingOpen()} title={["Welcome to Sparky", "Notifications", "Connect an agent"][onboardingStep()]!} onClose={finishOnboarding}
        footer={<><Button variant="ghost" onClick={finishOnboarding}>Skip onboarding</Button><Show when={onboardingStep() > 0}><Button onClick={() => setOnboardingStep(step => step - 1)}>Back</Button></Show>
          <Button onClick={() => onboardingStep() === 2 ? finishOnboarding() : setOnboardingStep(step => step + 1)}>{onboardingStep() === 2 ? "Get started" : "Continue"}</Button></>}>
        <Show when={onboardingStep() === 0}><DitherIconTile><Inbox size={24} /></DitherIconTile><p>Keep memories, plans, and Minds together on your Mac. Your data stays in the local core.</p></Show>
        <Show when={onboardingStep() === 1}><p>macOS delivers armed reminders even after Sparky closes. Reminders are refreshed when Sparky runs.</p>
          <StatusChip label={reminderLabel(core.status())} />
          <Show when={canRequestNotifications()} fallback={<p>Notification authorization is unavailable in this build. No reminder is armed by this step.</p>}>
            <Button disabled={authorizationBusy() || notificationGranted()} onClick={() => void enableNotifications()}>Enable notifications</Button>
            <Show when={authorizationError()}><p role="alert">{authorizationError()}</p></Show>
            <p>Enabling notifications does not arm a reminder. Armed reminders appear only after a live schedule.</p>
          </Show>
        </Show>
        <Show when={onboardingStep() === 2}><p>Connecting an AI coding agent is optional. Configure local MCP access in Settings → Agents → Access.</p>
          <Button onClick={() => { finishOnboarding(); const agents = settingsPanels().find(panel => panel.group.toLowerCase() === "agents"); if (agents) setSettingsPanel(agents.id); setSettingsOpen(true); }}>Open agent settings</Button>
        </Show>
      </Dialog>
      <Dialog open={helpOpen()} title="Sparky shortcuts" onClose={() => setHelpOpen(false)}>
        <p>⌘K Commander · ⌘N New memory · ⌘, Settings · ⌘[ Back · ⌘] Forward · ⌘J Close details</p>
        <p>⌘1 Inbox · ⌘2 Today · ⌘3 Calendar · ⌘4 Map · ⌘5 Focus · / Search</p>
        <p>In memory collections: Enter opens the selection, Space toggles completion, E edits the title, P pins, and ⌘⌫ asks to delete.</p>
      </Dialog>
    </DesignSystemProvider>
  </ShellContext.Provider>;
}
