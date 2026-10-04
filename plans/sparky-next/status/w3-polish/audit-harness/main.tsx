import { render } from 'solid-js/web';
import { fakeBridge as memories } from '../../../../../packages/features/sparky-memories/src/fake-bridge';
import { memoryFixture } from '../../../../../packages/features/sparky-memories/src/fixtures';
import { fakeSettingsBridge } from '../../../../../packages/features/sparky-settings/src/fake-bridge';
import { fakeBridge as focus } from '../../../../../packages/features/sparky-focus/src/fake-bridge';
import { fakeBridge as review } from '../../../../../packages/features/sparky-review/src/fake-bridge';
import { fakeBridge as search } from '../../../../../packages/features/sparky-search/src/fake-bridge';
import { fakeBridge as attachments } from '../../../../../packages/features/sparky-attachments/src/fake-bridge';
import { fakeMapBridge } from '../../../../../packages/features/sparky-map/src/fake-bridge';
import { fakeScheduleBridge } from '../../../../../packages/features/sparky-schedule/src/fake-bridge';
import { fakeCalendarBridge } from '../../../../../packages/features/sparky-calendar/src/fake-bridge';
import '../../../../../apps/sparky-desktop/src/renderer/styles.css';
import '../../../../../apps/sparky-desktop/node_modules/@fontsource-variable/inter/index.css';
const params = new URLSearchParams(location.search);
const state = params.get('state');
const seed = state === 'empty' ? [] : Array.from({ length: 8 }, (_, i) => ({ ...memoryFixture(i + 1), title: ['Read the design brief', 'Prepare the October review', 'Buy groceries', 'Plan a quiet afternoon to review every Memory with a deliberately long title that must remain readable at the minimum window size'][i % 4], dueAt: new Date(Date.now() + (i - 1) * 86400000).toISOString(), mindId: i > 4 ? 'mind-1' : '', pinned: i === 0 }));
const mem = memories(seed);
const listMemories = mem.bridge.memories.list;
const getMemory = mem.bridge.memories.get;
const settings = fakeSettingsBridge({ focusSounds: false, mapOnline: false });
const scheduled = fakeScheduleBridge({ ...seed[0], id: 'memory-1', status: 'active', rev: '1' });
const cal = fakeCalendarBridge({ occurrences: seed.map((m, i) => ({ memoryId: m.id!, title: m.title!, occurrenceAt: m.dueAt!, allDay: i % 2 === 0, tz: 'America/Sao_Paulo' })) });
const map = fakeMapBridge();
const wire: any = { ...settings.bridge, ...mem.bridge,
  settings: settings.bridge.settings, agents: settings.bridge.agents, maintenance: settings.bridge.maintenance,
  reminders: { ...settings.bridge.reminders, ...focus().bridge.reminders, status: async () => ({ permission: 'authorized', armed: 2, planned: 2, notifierLastSeenAt: new Date().toISOString() }) },
  focus: focus().bridge.focus, review: review().bridge.review,
  search: search().bridge.search, attachments: attachments([]).bridge.attachments,
  places: map.wire.places,
  schedule: { ...scheduled.wire.schedule, listOccurrences: cal.schedule.listOccurrences },
  onChange: () => () => {},
  app: { ...settings.bridge.app, onDeepLink: () => () => {}, getDeepLink: async () => params.get('route') ? { url: 'sparky://' + params.get('route'), route: '/' + params.get('route') } : null,
    openDeepLink: async () => {}, pickFiles: async () => [], revealInFinder: async () => {}, startMicrophone: async () => false },
};
// The list RPC exposes summaries, not MemoryDetail-only fields.
wire.memories.list = async (input: any = {}) => { const { memories: rows } = await listMemories(input); const filtered = (rows ?? []).filter(m => (!input.inboxOnly || !m.mindId) && (!input.mindId || m.mindId === input.mindId)); return { memories: filtered.map(({ checklist, tags, attachments, completions, ...m }) => m), total: filtered.length }; };
wire.memories.get = async (input: any) => { const result = await getMemory(input); return { memory: { ...result.memory, place: map.places.get(input.id) } }; };
wire.minds.list = async () => ({ minds: [{ id: 'mind-1', name: 'Home', color: '#8eccc9', icon: 'brain', rev: '1' }, { id: 'mind-2', parentId: 'mind-1', name: 'Projects with a long title to check truncation', color: '#5bbf93', rev: '1' }] });
wire.minds.get = async () => ({ mind: { id: 'mind-1', name: 'Home', color: '#8eccc9', rev: '1' } });
if (state === 'offline') wire.reminders.status = async () => { throw new Error('The local core is unreachable.'); };
if (state === 'loading') wire.memories.list = () => new Promise(() => {});
if (state === 'error') { const list = wire.memories.list; wire.memories.list = async (input: any = {}) => { if ((input.limit ?? 500) > 1) throw new Error('Could not load memories. Retry to reconnect.'); return list(input); }; }
Reflect.set(window, 'sparky', wire);
if (params.get('onboarding') === 'true') localStorage.removeItem('sparky.shell.onboarding'); else localStorage.setItem('sparky.shell.onboarding', 'complete');
localStorage.setItem('sparky.map.disclosure', 'seen');
document.documentElement.dataset.shellChrome = params.get('solid') === 'true' ? 'solid' : 'vibrancy';
const { SparkyShell, createShellBridge } = await import('../../../../../packages/features/sparky-shell/src/index');
const { features: registeredFeatures } = await import('../../../../../apps/sparky-desktop/src/renderer/features');
const { ErrorBoundary } = await import('solid-js');
const { MemoryCollection } = await import('../../../../../packages/features/sparky-memories/src/memory-list');
// Explicit component previews only: never count these as integrated behavior evidence.
const features = registeredFeatures.map(feature => params.get('preview') === 'map-settings' && feature.id === 'map' ? { ...feature, settingsPanels: [] } : params.get('preview') === 'list' && feature.id === 'memories' ? { ...feature, routes: feature.routes.map(route => route.path === '/today' || route.path === '/inbox' ? { ...route, component: () => <MemoryCollection mode={route.path === '/today' ? 'today' : 'inbox'} virtualized={false} /> } : route) } : feature);
const App = () => <ErrorBoundary fallback={error => { console.error(error.stack); return <pre>{error.stack}</pre>; }}><SparkyShell features={features} bridge={createShellBridge(wire)} /></ErrorBoundary>;
render(() => <App />, document.getElementById('root')!);
