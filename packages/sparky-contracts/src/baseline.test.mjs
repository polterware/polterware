import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { describe, expect, it } from "vitest";

const sqlPath = new URL("../../../sparky/backend/providers/sqlite/migrations/0001_baseline.sql", import.meta.url);

describe("SQL baseline", () => {
  const sql = readFileSync(sqlPath, "utf8");

  it("applies to an empty database and foreign_key_check is clean", () => {
    const db = new DatabaseSync(":memory:");
    db.exec("PRAGMA foreign_keys = ON");
    db.exec(sql);
    expect(db.prepare("PRAGMA foreign_key_check").all()).toEqual([]);

    const now = "2026-10-03T21:30:00.000Z";
    db.prepare("INSERT INTO minds (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)").run(
      "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "Work",
      now,
      now,
    );
    db.prepare("INSERT INTO memories (id, mind_id, title, created_at, updated_at) VALUES (?, ?, ?, ?, ?)").run(
      "01ARZ3NDEKTSV4RRFFQ69G5FAW",
      "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "Buy milk",
      now,
      now,
    );
    db.prepare("DELETE FROM minds WHERE id = ?").run("01ARZ3NDEKTSV4RRFFQ69G5FAV");
    const memory = db.prepare("SELECT mind_id FROM memories WHERE id = ?").get("01ARZ3NDEKTSV4RRFFQ69G5FAW");
    expect(memory.mind_id).toBeNull();
    expect(db.prepare("PRAGMA foreign_key_check").all()).toEqual([]);
  });
});
