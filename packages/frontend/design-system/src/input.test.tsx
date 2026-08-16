import { describe, expect, it } from "vitest";
import { Input, Textarea } from "./input";
import { Select } from "./select";
import { Field } from "./field";
import { MenuItem, MenuPanel } from "./menu";

describe("form and menu primitives", () => {
  it("exports Input / Select / Textarea / Field / Menu", () => {
    expect(Input).toBeDefined();
    expect(Select).toBeDefined();
    expect(Textarea).toBeDefined();
    expect(Field).toBeDefined();
    expect(MenuPanel).toBeDefined();
    expect(MenuItem).toBeDefined();
  });
});
