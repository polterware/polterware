#!/usr/bin/env node
// Seeds a scratch Polterware database for the Desktop comparisons (tooling/bench, tooling/parity):
// the same projects, structure, milestones, cards, decisions, rules, docs, conflicts, and change
// history every time, so the Electron and GPUI Desktops read identical data. It starts its own
// core on the database and writes through the same Connect RPCs the Desktop uses.
//
//   node tooling/bench/seed.mjs [--db dist/bench/typical/local.db] [--size typical|large]
//                               [--bin dist/polterware]
//
// The database must not exist yet; the user's own database is refused.
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, arg, i, all) => {
    if (arg.startsWith("--")) pairs.push([arg.slice(2), all[i + 1]?.startsWith("--") ? "" : all[i + 1] ?? ""]);
    return pairs;
  }, []),
);
const size = args.size || "typical";
const db = path.resolve(args.db || path.join(root, "dist/bench", size, "local.db"));
const bin = path.resolve(args.bin || path.join(root, "dist/polterware"));

const SIZES = {
  // A project people work in every day, plus two small ones for the Projects screen.
  typical: { cards: 120, tasks: 5, decisions: 30, rules: 20, docs: 40, conflicts: 6, questions: 8, updates: 600, others: 2 },
  // A long-lived project: boards, lists, and docs big enough to show what scrolling and memory
  // cost. Activity reads only the latest 50 changes in both Desktops, so history stays modest.
  large: { cards: 2000, tasks: 6, decisions: 300, rules: 200, docs: 300, conflicts: 12, questions: 20, updates: 3000, others: 4 },
};
const plan = SIZES[size] ?? fail(`unknown --size ${size}`);

let core;
function fail(message) {
  console.error(`seed: ${message}`);
  core?.kill();
  process.exit(1);
}

const userDb = path.join(os.homedir(), "Library/Application Support/polterware/local.db");
if (db === userDb) fail("refusing to seed the user's own database");
if (existsSync(db)) fail(`${db} exists; remove its folder to seed again`);
if (!existsSync(bin)) fail(`${bin} is missing; run pnpm build:cli`);

// Deterministic content: a small PRNG with a fixed seed.
let state = 0x5eed;
const random = () => ((state = (state * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);
const pick = (list) => list[Math.floor(random() * list.length)];
const WORDS = (
  "cache index import export sync queue retry token session ledger snapshot cursor batch shard " +
  "schema migration webhook audit report billing invoice search filter rollout feature flag " +
  "metrics trace alert deploy preview build release review handoff context brief canvas board"
).split(" ");
const VERBS = "Add Wire Split Remove Tune Document Harden Cache Measure Rename Move Validate Stream Batch".split(" ");
const title = () => `${pick(VERBS)} ${pick(WORDS)} ${pick(WORDS)}`;
const sentence = (n = 12) => {
  const words = Array.from({ length: n }, () => pick(WORDS));
  return `${words.join(" ").replace(/^./, (c) => c.toUpperCase())}.`;
};
const STAGES = ["shape", "plan", "build", "verify", "delivered"];

const dir = path.dirname(db);
mkdirSync(dir, { recursive: true });
core = spawn(bin, ["-db", db, "serve", "--listen", "127.0.0.1:0"], { stdio: ["ignore", "ignore", "inherit"] });
core.on("exit", (code) => {
  if (code && !finished) fail(`the core exited with ${code}`);
});
let finished = false;

async function connect() {
  const file = path.join(dir, "run/core.json");
  for (let i = 0; i < 150; i++) {
    if (existsSync(file)) {
      try {
        const { endpoint, token } = JSON.parse(readFileSync(file, "utf8"));
        if (endpoint && token) return { endpoint: endpoint.replace(/\/$/, ""), token };
      } catch {}
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  fail("the core did not start in 15 seconds");
}

const { endpoint, token } = await connect();
let calls = 0;
async function rpc(service, method, body) {
  calls++;
  const response = await fetch(`${endpoint}/polterware.local.v1.${service}/${method}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Connect-Protocol-Version": "1",
      Authorization: `Bearer ${token}`,
      "Polterware-Client": "seed",
    },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  if (!response.ok) fail(`${service}.${method}: ${response.status} ${text}`);
  return text ? JSON.parse(text) : {};
}
const change = (summary, rationale = "Seed data for the Desktop comparison.") => ({ summary, rationale });
async function apply(projectId, text, summary) {
  const header = `---\npolter: 1\nsummary: ${summary}\nrationale: Seed data for the Desktop comparison.\n---\n\n`;
  const result = await rpc("WorkService", "ApplyPolter", { projectId, text: header + text, change: change(summary) });
  const errors = (result.diagnostics ?? []).filter((d) => d.severity === "error");
  if (!result.applied || errors.length) fail(`apply "${summary}": ${JSON.stringify(result.diagnostics ?? result)}`);
  return result;
}

const started = Date.now();

async function seedProject(name, scale) {
  const { project } = await rpc("ProjectService", "InitProject", { name, dir: "" });
  const id = project.id;
  const count = (n) => Math.max(1, Math.round(n * scale));

  // Structure: four domains, three components each.
  const domains = ["core", "app", "data", "ops"];
  const components = domains.flatMap((domain) => [0, 1, 2].map((i) => `${domain}/${pick(WORDS)}-${i}`));
  const kinds = { core: "service", app: "frontend", data: "database", ops: "worker" };
  const structure = [
    ...domains.map((domain) => `# ${domain}: ${domain[0].toUpperCase()}${domain.slice(1)}\nlevel: domain\n\n${sentence()}\n`),
    ...components.map((node) => {
      const domain = node.split("/")[0];
      return `# ${node}: ${title()}\nlevel: component · under: ${domain} · kind: ${kinds[domain]} · tech: ${pick(["Go", "Rust", "TypeScript", "Postgres", "Redis"])}\n\n${sentence()}\n`;
    }),
  ].join("\n");
  await apply(id, structure, "Lay out the project structure");

  const milestones = [];
  for (const [i, state] of ["done", "active", "planned"].entries()) {
    const { milestone } = await rpc("WorkService", "CreateMilestone", {
      projectId: id,
      name: `${["Alpha", "Beta", "General availability"][i]}`,
      goal: sentence(10),
      exitCriteria: [sentence(6), sentence(6)],
      targetDate: ["2026-09-15", "2026-11-30", "2027-02-15"][i],
      state,
      change: change("Plan a milestone"),
    });
    milestones.push(milestone.slug);
  }

  // Cards, one apply each, so each is its own change.
  const cards = [];
  for (let i = 0; i < count(plan.cards); i++) {
    const slug = `card-${String(i + 1).padStart(4, "0")}`;
    const stage = pick(STAGES);
    const done = Array.from({ length: plan.tasks }, () => stage === "delivered" || random() < 0.4);
    const tasks = done.map((isDone, t) => `- [${isDone ? "x" : " "}] T${t + 1} ${title()}`);
    const milestone = random() < 0.8 ? ` · milestone: ${pick(milestones)}` : "";
    cards.push({ slug, stage, done });
    await apply(
      id,
      `## ${slug}: ${title()}\nstage: ${stage} · under: ${pick(components)}${milestone}\n\nIntent: ${sentence(18)}\n\n${tasks.join("\n")}\n! ${sentence(8)}\n`,
      `Add ${slug}`,
    );
  }
  // Open questions on cards: Inbox Questions.
  for (let i = 0; i < count(plan.questions); i++) {
    const card = cards[i % cards.length];
    await apply(id, `## ${card.slug}\n? ${title()}?${i % 3 === 0 ? " [blocking]" : ""} (yes | no)\n`, `Ask on ${card.slug}`);
  }

  for (let i = 0; i < count(plan.decisions); i++) {
    const scope = random() < 0.25 ? "" : ` · scope: ${pick(components)}`;
    await apply(id, `### new: ${title()}\nstatus: ${pick(["active", "active", "proposed"])}${scope}\n\n${sentence(30)}\n`, "Record a decision");
  }
  const rules = [];
  for (let i = 0; i < count(plan.rules); i++) {
    const slug = `rule-${i + 1}`;
    rules.push(slug);
    const scope = random() < 0.3 ? "" : `\nscope: ${pick(components)}`;
    await apply(id, `#### rule ${slug}: ${pick(VERBS)} every ${pick(WORDS)} before the ${pick(WORDS)}.${scope}\n`, `Add ${slug}`);
  }
  // Conflicts against rules: Inbox Conflicts.
  for (let i = 0; i < count(plan.conflicts); i++) {
    await apply(
      id,
      `#### conflict conflict-${i + 1}: ${title()}\nrule: ${rules[i % rules.length]}\ncard: ${cards[i % cards.length].slug}\noptions: Keep the rule | Allow it for this card\n`,
      "Raise a conflict",
    );
  }
  for (let i = 0; i < count(plan.docs); i++) {
    const body = Array.from({ length: 4 }, () => `## ${title()}\n\n${sentence(40)}\n\n- ${sentence(8)}\n- ${sentence(8)}`).join("\n\n");
    await rpc("KnowledgeService", "SaveKnowledge", {
      item: {
        // The client names a new item, as the Desktop does (`know-<hex>`).
        id: `know-${Array.from({ length: 16 }, () => Math.floor(random() * 16).toString(16)).join("")}`,
        projectId: id,
        kind: "doc",
        docType: "doc",
        title: title(),
        body,
        tags: [pick(WORDS), pick(WORDS)],
      },
      change: change("Write a doc"),
    });
  }
  // History: a task checked or opened again, or the card moved a stage; each one a real change.
  for (let i = 0; i < count(plan.updates); i++) {
    const card = pick(cards);
    let text;
    if (random() < 0.7) {
      const t = Math.floor(random() * plan.tasks);
      card.done[t] = !card.done[t];
      text = `## ${card.slug}\n- [${card.done[t] ? "x" : " "}] T${t + 1}\n`;
    } else {
      const next = card.stage === "delivered" ? "build" : STAGES[STAGES.indexOf(card.stage) + 1];
      card.stage = next;
      text = `## ${card.slug}\nstage: ${next}\n`;
    }
    await apply(id, text, `Update ${card.slug}`);
  }
  return id;
}

const main = await seedProject("Atlas", 1);
for (const name of ["Beacon", "Comet", "Delta", "Echo"].slice(0, plan.others)) await seedProject(name, 0.15);

const inbox = await rpc("WorkService", "ListInbox", { projectId: main });
const history = await rpc("AgentRunService", "ListChanges", { projectId: main, limit: 100000 }).catch(() => null);
finished = true;
await rpc("MaintenanceService", "Shutdown", { reason: "seed done" }).catch(() => core.kill());
console.log(
  JSON.stringify({
    db,
    size,
    calls,
    seconds: Math.round((Date.now() - started) / 100) / 10,
    inbox: inbox.items?.length ?? 0,
    changes: history?.changes?.length ?? null,
  }),
);
