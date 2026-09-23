export type PpmFactTruthClass = "declared" | "observed" | "inferred" | "verified";

export type PpmFactRow = {
  id: string;
  projectId: string;
  statement: string;
  truthClass: string;
  source: string;
  actor: string;
  entityId: string;
  observedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type PpmEvidenceRow = {
  id: string;
  projectId: string;
  kind: string;
  summary: string;
  source: string;
  path: string;
  actor: string;
  observedAt: string;
  createdAt: string;
  updatedAt: string;
};

export type PpmFactDetail = {
  fact: PpmFactRow;
  evidence: PpmEvidenceRow[];
};

export type PpmFactDraft = {
  id?: string;
  statement: string;
  truthClass: PpmFactTruthClass;
  entityId: string;
};

export type PpmEvidenceDraft = {
  kind: string;
  summary: string;
  path: string;
  linkToSelectedFact: boolean;
};

export const PPM_TRUTH_CLASS_OPTIONS: Array<{ value: PpmFactTruthClass; label: string }> = [
  { value: "declared", label: "Declared" },
  { value: "observed", label: "Observed" },
  { value: "inferred", label: "Inferred" },
  { value: "verified", label: "Verified" },
];

export const PPM_EVIDENCE_KIND_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "file", label: "File" },
  { value: "observation", label: "Observation" },
  { value: "test", label: "Test" },
  { value: "commit", label: "Commit" },
  { value: "other", label: "Other" },
];
