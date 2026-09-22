export type DataSource = "fixture" | "empty" | "unavailable";

export type EnvironmentKind = "production" | "staging" | "development" | "local";

export type ProtectionLevel = "strict" | "standard" | "relaxed";

export type ProjectEnvironment = {
  id: string;
  name: string;
  kind: EnvironmentKind;
  protection: ProtectionLevel;
  resourceCount: number;
  nodeCount: number;
  isDefault: boolean;
  /** Fixture summary lines for this local environment. */
  protectionRules: string[];
  description?: string;
  dataSource: DataSource;
};

export type ProjectGeneralSettings = {
  name: string;
  description: string;
  defaultEnvironmentId: string;
  projectId: string;
  repository: string;
  dataSource: DataSource;
};

export type SettingsTabId = "general" | "environments" | "shortcuts" | "advanced";
