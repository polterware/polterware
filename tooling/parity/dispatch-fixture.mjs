// A stand-in Polterware Dispatch core for the parity tour and the bench, so both Desktops read
// the same Dispatch history and neither starts the Dispatch app installed on this computer.
// It answers HealthService/Check and DispatchService/ListHistory in Connect's unary protocol,
// as JSON or binary, and names itself in <data dir>/run/core.json the way `dispatch serve`
// does. Every project gets the same few runs, timed back from when the fixture started, so
// the two apps, captured one after the other, show the same times.
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import http from "node:http";
import path from "node:path";

const MINUTE = 60_000;

/** The runs Activity lists: started, merged, and failed, by two agents on three tiers. */
function history(startedAt) {
  const at = (minutes) => new Date(startedAt - minutes * MINUTE).toISOString();
  return [
    { id: "evt-0004", runId: "run-0b12", kind: "merged", agent: "codex", tier: "normal", text: "Wire the export queue to the CLI", seq: 4, at: at(2) },
    { id: "evt-0003", runId: "run-0b11", kind: "failed", agent: "claude", tier: "hard", text: "Tests failed in sync/worker_test.go", seq: 3, at: at(7) },
    { id: "evt-0002", runId: "run-0b11", kind: "started", agent: "claude", tier: "hard", text: "Split the sync worker", seq: 2, at: at(9) },
    { id: "evt-0001", runId: "run-0b10", kind: "started", agent: "codex", tier: "easy", text: "", seq: 1, at: at(12) },
  ];
}

// Protobuf's wire format, for the clients that ask for application/proto.
const varint = (value) => {
  const bytes = [];
  let rest = BigInt(value);
  while (rest > 127n) {
    bytes.push(Number(rest & 127n) | 128);
    rest >>= 7n;
  }
  bytes.push(Number(rest));
  return Buffer.from(bytes);
};
const tag = (field, wire) => varint((field << 3) | wire);
const text = (field, value) => {
  const bytes = Buffer.from(value ?? "", "utf8");
  return bytes.length ? Buffer.concat([tag(field, 2), varint(bytes.length), bytes]) : Buffer.alloc(0);
};
const int = (field, value) => (value ? Buffer.concat([tag(field, 0), varint(value)]) : Buffer.alloc(0));
const nested = (field, body) => Buffer.concat([tag(field, 2), varint(body.length), body]);
const binaryEvent = (event) =>
  Buffer.concat([
    text(1, event.id),
    text(2, event.runId),
    text(3, event.projectId),
    text(4, event.kind),
    text(5, event.agent),
    text(6, event.tier),
    text(7, event.text),
    int(8, event.seq),
    text(9, event.at),
  ]);

/** ListHistoryRequest's project_id (field 1) from a binary request. */
function binaryProject(body) {
  let offset = 0;
  while (offset < body.length) {
    let key = 0;
    let shift = 0;
    let byte;
    do {
      byte = body[offset++];
      key |= (byte & 127) << shift;
      shift += 7;
    } while (byte & 128);
    const wire = key & 7;
    if (wire !== 2) {
      // A varint field: skip it.
      while (body[offset++] & 128);
      continue;
    }
    let length = 0;
    shift = 0;
    do {
      byte = body[offset++];
      length |= (byte & 127) << shift;
      shift += 7;
    } while (byte & 128);
    if (key >> 3 === 1) return body.subarray(offset, offset + length).toString("utf8");
    offset += length;
  }
  return "";
}

/** Starts the fixture under `dataDir`; `env` points both apps at it, `close` takes it down. */
export async function startDispatchFixture(dataDir) {
  const startedAt = Date.now();
  const token = "parity-fixture";
  const server = http.createServer((request, response) => {
    const chunks = [];
    request.on("data", (chunk) => chunks.push(chunk));
    request.on("end", () => {
      const body = Buffer.concat(chunks);
      const binary = (request.headers["content-type"] ?? "").includes("proto");
      const reply = (json, proto) => {
        response.writeHead(200, { "Content-Type": binary ? "application/proto" : "application/json" });
        response.end(binary ? proto : JSON.stringify(json));
      };
      switch (request.url) {
        case "/dispatch.v1.HealthService/Check":
          return reply({ status: "ok", pid: process.pid }, Buffer.concat([text(1, "ok"), int(5, process.pid)]));
        case "/dispatch.v1.DispatchService/ListHistory": {
          if (request.headers.authorization !== `Bearer ${token}`) break;
          let projectId = "";
          try {
            projectId = binary ? binaryProject(body) : (JSON.parse(body.toString("utf8") || "{}").projectId ?? "");
          } catch {}
          const events = history(startedAt).map((event) => ({ ...event, projectId }));
          return reply(
            { events: events.map((event) => ({ ...event, seq: String(event.seq) })) },
            Buffer.concat(events.map((event) => nested(1, binaryEvent(event)))),
          );
        }
      }
      response.writeHead(404, { "Content-Type": "application/json" });
      response.end(JSON.stringify({ code: "unimplemented", message: `${request.url} is not in the fixture` }));
    });
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const endpoint = `http://127.0.0.1:${server.address().port}`;
  const run = path.join(dataDir, "run");
  mkdirSync(run, { recursive: true });
  writeFileSync(path.join(run, "core.json"), `${JSON.stringify({ pid: process.pid, endpoint, token })}\n`, { mode: 0o600 });
  return {
    env: { PWDISPATCH_DATA_DIR: dataDir },
    close() {
      rmSync(path.join(run, "core.json"), { force: true });
      server.closeAllConnections();
      server.close();
    },
  };
}
