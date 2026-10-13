export const CARD_IDS = ["cpu", "memory", "network", "diskio", "pressure", "disk", "thermal", "processes"] as const;

export type CardId = (typeof CARD_IDS)[number];
