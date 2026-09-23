import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { Input, Textarea } from "./input";
import { Select } from "./select";
import { Field } from "./field";
import { MenuItem, MenuPanel } from "./menu";

const inputSourcePath = resolve(fileURLToPath(new URL(".", import.meta.url)), "./input.tsx");

describe("form and menu primitives", () => {
  it("exports Input / Select / Textarea / Field / Menu", () => {
    expect(Input).toBeDefined();
    expect(Select).toBeDefined();
    expect(Textarea).toBeDefined();
    expect(Field).toBeDefined();
    expect(MenuPanel).toBeDefined();
    expect(MenuItem).toBeDefined();
  });

  it("defaults Input and Textarea to spellcheck off and autocomplete off", () => {
    const source = readFileSync(inputSourcePath, "utf8");
    expect(source).toMatch(/spellcheck=\{local\.spellcheck \?\? false\}/);
    expect(source).toMatch(/autocomplete=\{local\.autocomplete \?\? "off"\}/);
    expect(source.match(/spellcheck=\{local\.spellcheck \?\? false\}/g)?.length).toBe(2);
    expect(source.match(/autocomplete=\{local\.autocomplete \?\? "off"\}/g)?.length).toBe(2);
  });
});
