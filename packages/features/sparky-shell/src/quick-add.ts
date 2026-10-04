import { fromJson, toJson } from "@bufbuild/protobuf";
import { CreateMemoryRequestSchema } from "@polterware/sparky-contracts";
import type { ShellBridge } from "./bridge";

export type QuickAdd = { title: string; tags: string[]; mind?: string; priority?: number; dueAt?: string; dueAllDay: boolean };
const weekdays = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];
/** English, local wall-clock dates. Unknown text stays in the title; ambiguous input fails visibly. */
export function parseQuickAdd(input: string, now = new Date()): QuickAdd {
  let text = input.trim();
  const result: QuickAdd = { title: "", tags: [], dueAllDay: false };
  text = text.replace(/(?:^|\s)#([\p{L}\p{N}_-]+)/gu, (_, tag: string) => { result.tags.push(tag); return " "; });
  text = text.replace(/(?:^|\s)@(?:"([^"]+)"|([\p{L}\p{N}_-]+))/gu, (_, quoted: string, word: string) => {
    if (result.mind) throw new Error("Use one @Mind per memory.");
    result.mind = quoted ?? word; return " ";
  });
  text = text.replace(/(?:^|\s)(!{1,3})(?=\s|$)/g, (_, marks: string) => {
    if (result.priority) throw new Error("Use one priority marker: !, !! or !!!.");
    result.priority = marks.length; return " ";
  });
  let due: Date | undefined;
  let relative = false;
  text = text.replace(/\bin\s+(\d+)\s+(hours?|minutes?)\b/gi, (_, amount: string, unit: string) => {
    if (due) throw new Error("Use one due date per memory.");
    const minutes = Number(amount) * (unit.toLowerCase().startsWith("hour") ? 60 : 1);
    if (minutes <= 0 || minutes > 525600) throw new Error("Use a duration between 1 minute and 1 year.");
    due = new Date(now.getTime() + minutes * 60000); relative = true; return " ";
  });
  text = text.replace(/\b(today|tomorrow|sunday|monday|tuesday|wednesday|thursday|friday|saturday)\b/gi, (_, word: string) => {
    if (due) throw new Error("Use one due date per memory.");
    due = new Date(now); due.setHours(0, 0, 0, 0);
    const day = word.toLowerCase();
    const offset = day === "today" ? 0 : day === "tomorrow" ? 1 : (weekdays.indexOf(day) - now.getDay() + 7) % 7 || 7;
    due.setDate(due.getDate() + offset); result.dueAllDay = true; return " ";
  });
  let timeSeen = false;
  text = text.replace(/\b(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b|\b(\d{1,2}):(\d{2})\b/gi, (_, hour12: string, minute12: string, meridiem: string, hour24: string, minute24: string) => {
    if (relative || timeSeen) throw new Error("Use one time per memory.");
    let hour = Number(hour12 ?? hour24); const minute = Number(minute12 ?? minute24 ?? 0);
    if (minute > 59 || (meridiem ? hour < 1 || hour > 12 : hour > 23)) throw new Error("Use a valid time, such as 9am or 14:30.");
    if (meridiem) hour = hour % 12 + (meridiem.toLowerCase() === "pm" ? 12 : 0);
    const explicitDate = !!due;
    due ??= new Date(now); due.setHours(hour, minute, 0, 0);
    if (due.getHours() !== hour || due.getMinutes() !== minute) throw new Error("That local time does not exist due to daylight saving time.");
    if (!explicitDate && due <= now) due.setDate(due.getDate() + 1);
    result.dueAllDay = false; timeSeen = true; return " ";
  });
  result.title = text.replace(/\s+/g, " ").trim();
  if (!result.title) throw new Error("Enter a memory title.");
  result.tags = [...new Map(result.tags.map(tag => [tag.toLowerCase(), tag])).values()];
  result.dueAt = due?.toISOString();
  return result;
}

/** Resolve metadata before creating the memory, then submit one Create payload. */
export async function createQuickMemory(bridge: ShellBridge, parsed: QuickAdd) {
  let mindId: string | undefined;
  if (parsed.mind) {
    const { minds = [] } = await bridge.minds.list({});
    const matches = minds.filter(mind => mind.name?.toLowerCase() === parsed.mind!.toLowerCase());
    if (matches.length !== 1 || !matches[0]?.id) throw new Error(matches.length ? `@${parsed.mind} is ambiguous. Choose a unique Mind name.` : `Mind @${parsed.mind} was not found.`);
    mindId = matches[0].id;
  }
  const tagIds: string[] = [];
  if (parsed.tags.length) {
    const { tags = [] } = await bridge.tags.list({});
    for (const name of parsed.tags) {
      let tag = tags.find(candidate => candidate.name?.toLowerCase() === name.toLowerCase());
      if (!tag) tag = (await bridge.tags.create({ name })).tag;
      if (!tag?.id) throw new Error(`Could not resolve #${name}.`);
      tagIds.push(tag.id);
    }
  }
  const input = toJson(CreateMemoryRequestSchema, fromJson(CreateMemoryRequestSchema, {
    title: parsed.title, tagIds, dueAllDay: parsed.dueAllDay,
    ...(mindId ? { mindId } : {}),
    ...(parsed.priority ? { priority: parsed.priority } : {}),
    ...(parsed.dueAt ? { dueAt: parsed.dueAt } : {}),
  }));
  const { memory } = await bridge.memories.create(input);
  if (!memory?.id) throw new Error("The core did not return the created memory.");
  return memory;
}
