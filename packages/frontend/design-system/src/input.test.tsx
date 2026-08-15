import { describe, expect, it } from "vitest";
import { Input, Textarea } from "./input";
import { Select } from "./select";
import { Field } from "./field";
import { MenuItem, MenuPanel } from "./menu";

describe("form and menu primitives", () => {
  it("exports Input / Select / Textarea / Field / Menu", () => {
    expect(typeof Input).toBe("function");
    expect(typeof Select).toBe("function");
    expect(typeof Textarea).toBe("function");
    expect(typeof Field).toBe("function");
    expect(typeof MenuPanel).toBe("function");
    expect(typeof MenuItem).toBe("function");
  });
});
