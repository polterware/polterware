# Swift / Bun reference map (read-only)

Root of the old app: `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky` (Swift; sources under `sparky/sparky/` and
`sparky/sparkyMac/`; specs under `specs/001…005`; docs under `docs/`). Old MCP: `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky-mcp`.
Use these to understand **behavior and intent**. Do not port code literally, and do not edit anything there.

| Topic | Swift/Bun files (relative to the roots above) | Used by |
|-------|-----------------------------------------------|---------|
| Memory model, status, completion | `sparky/Model/Memory/{Memory,MemoryStatus,MemoryCompletionDate,MemoryAttachmentReference,MemoryDraft}.swift` | W0-CONTRACTS, W1-DOMAIN |
| Mind hierarchy | `sparky/Model/Mind/Mind.swift`, `Services/MindService.swift`, `Views/Minds/*` | W1-DOMAIN, W2-MINDS |
| Checklist | `sparky/Model/CheckItem/*`, `Views/Memories/Editor/Synapse/*`, `Views/Memories/Editor/MemoryEditorChecklistCard.swift` | W2-MEMORIES |
| Schedule, recurrence | `sparky/Model/Triggers/{ScheduleConfig,ScheduleConfigDraft,TimeOfDayType}.swift`, `Model/Recurrence/*` | W1-RECURRENCE |
| Reminder planning | `sparky/Executors/{ScheduledTriggerPlanner,ScheduledTriggerExecutor,TriggerExecutorCoordinator}.swift`, `Services/NotificationCategoryID.swift`, `Settings/MemoryNotificationSound.swift` | W0/W1-NOTIFIER, W1-RECURRENCE |
| Schedule editor UI | `Views/Memories/Editor/Triggers/**` | W2-SCHEDULE |
| Location / map | `Model/Triggers/{LocationConfig,LocationEvent}.swift`, `Views/Memories/Map/*`, `Views/Memories/Editor/Triggers/Location/*`, `Executors/Location*` (do **not** port geofence arming) | W2-MAP |
| Focus | `Focus/*` (FocusTimer, FocusRecipe, FocusSettings, FocusPresetOptions, sounds, notifications), `Views/Focus/*`, `specs/001-focus-tab-pomodoro`, `specs/002-focus-screen-redesign` | W2-FOCUS |
| Calendar | `Views/Desktop/Calendar/*`, `Views/Memories/Calendar/*`, `specs/004-desktop-calendar-redesign` | W2-CALENDAR |
| Review ("Me") | `Views/Settings/{MeView,MeViewModel,MeMetrics,MeMetrics+Presentation,MeCharacterMood,MeProgressMilestone,WeeklyActivityCard,ContributionCalendarCard,ContributionCalendarLayout}.swift` | W1-DOMAIN, W2-REVIEW |
| Attachments | `Managers/MemoryAttachmentStore.swift`, `Views/Memories/Editor/Components/*` (photo, audio, link, file) | W1-ATTACH, W2-ATTACH-UI |
| Editor layout & behavior | `ViewModels/MemoryEditorViewModel.swift`, `Views/Memories/Editor/*`, `Views/Desktop/DesktopMemoryEditorPopover.swift` | W2-MEMORIES |
| Desktop navigation | `Views/Desktop/{DesktopRootView,DesktopNavigationState,DesktopMainWindow,DesktopFloatingNavigationBar}.swift`, `specs/003-desktop-multiplatform` | W2-SHELL |
| Search / filters | `Views/Memories/MemorySearchSheet.swift`, `Views/Desktop/DesktopMemorySearchPopover.swift`, `Views/Shared/Filter/*`, `Model/Memory/MemoryContentFilterType.swift`, `MemoryTimelineFilter.swift` | W2-SEARCH-UI |
| Settings | `Views/Settings/{SettingsView,NotificationSettingsView,FocusSettingsView,ThemeSettingsView,AdvancedSettingsView}.swift`, `Settings/SettingsStore.swift` | W2-SETTINGS |
| Onboarding | `Views/Onboarding/OnboardingFlowView.swift` | W2-SHELL |
| Export formats (data shape reference only) | `Model/Export/{SparkyExportFormat,ICalExportFormat}.swift`, `Services/DataExportService.swift` | W1-CORE (`MaintenanceService.Export`) |
| Old MCP contract and tools | `sparky-mcp/docs/CONTRACT.md`, `sparky-mcp/src/{mcp,commands,schema,mirror}.ts`, `sparky-mcp/test/**` | W0-CONTRACTS, W1-MCP |
| Old MCP agent guide | `sparky-mcp/docs/GUIDE.md` | W2-SETTINGS (Agents copy) |
| Character art | `sparky-mcp/src/assets/sparky-character.webp`, Swift asset catalogs (`*.xcassets`) | W1-ASSETS |
| Icons | `new.icon/` (new), `sparky/AppIcon*.icon` (old variants) | W1-ASSETS |

Things in the Swift app that are **intentionally not carried over**: iPhone UI and tab bar, geofence arming, SwiftData/CloudKit
details, remote sync/pairing (`Remote/*`, `RemoteMCPSettingsSection`, `MCPSettingsView`), Sparkle/appcast, AppIconManager
(alternate icons), haptics, Liquid Glass modifiers.
