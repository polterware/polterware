import { createClient, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-node";
import { MemoryService, MindService, TagService, ScheduleService, PlaceService, AttachmentService, FocusService,
  ReviewService, SearchService, ReminderService, ActivityService, SettingsService, AgentAccessService,
  MaintenanceService, HealthService, ChangeService } from "@polterware/sparky-contracts";

export const services = {
  memories: MemoryService, minds: MindService, tags: TagService, schedule: ScheduleService,
  places: PlaceService, attachments: AttachmentService, focus: FocusService, review: ReviewService,
  search: SearchService, reminders: ReminderService, activity: ActivityService, settings: SettingsService,
  agents: AgentAccessService, maintenance: MaintenanceService,
} as const;
export function validateEndpoint(endpoint: string): string {
  const url = new URL(endpoint);
  if (url.protocol !== "http:" || url.hostname !== "127.0.0.1" || !url.port || url.username || url.password ||
      url.pathname !== "/" || url.search || url.hash) throw new Error("Core endpoint must be IPv4 loopback HTTP");
  return url.origin;
}
export function createCoreClients(endpoint: string, token: string) {
  const auth: Interceptor = next => async request => {
    request.header.set("Authorization", `Bearer ${token}`);
    request.header.set("Sparky-Client", "desktop");
    return next(request);
  };
  const transport = createConnectTransport({ baseUrl: validateEndpoint(endpoint), httpVersion: "1.1", interceptors: [auth] });
  return {
    memories: createClient(MemoryService, transport), minds: createClient(MindService, transport),
    tags: createClient(TagService, transport), schedule: createClient(ScheduleService, transport),
    places: createClient(PlaceService, transport), attachments: createClient(AttachmentService, transport),
    focus: createClient(FocusService, transport), review: createClient(ReviewService, transport),
    search: createClient(SearchService, transport), reminders: createClient(ReminderService, transport),
    activity: createClient(ActivityService, transport), settings: createClient(SettingsService, transport),
    agents: createClient(AgentAccessService, transport), maintenance: createClient(MaintenanceService, transport),
    health: createClient(HealthService, transport), changes: createClient(ChangeService, transport),
  };
}
export type CoreClients = ReturnType<typeof createCoreClients>;
