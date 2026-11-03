import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "../document/document";
import { addNode, addRelationship } from "../document/mutations";
import { hitTest } from "./hit-test";

describe("hitTest", () => {
  it("detects four invisible hit zones on connectable nodes", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = addNode(doc, {
      id: "node",
      name: "Node",
      type: "service",
      x: 100,
      y: 100,
      width: 100,
      height: 100,
    });

    expect(hitTest(doc, { x: 100, y: 150 })).toMatchObject({
      kind: "port",
      nodeId: "node",
      portId: "left",
    });
    expect(hitTest(doc, { x: 200, y: 150 })).toMatchObject({
      kind: "port",
      nodeId: "node",
      portId: "right",
    });
    expect(hitTest(doc, { x: 150, y: 100 })).toMatchObject({
      kind: "port",
      nodeId: "node",
      portId: "top",
    });
    expect(hitTest(doc, { x: 150, y: 200 })).toMatchObject({
      kind: "port",
      nodeId: "node",
      portId: "bottom",
    });
  });

  it("treats an empty ports array as unconnectable", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = addNode(doc, {
      id: "note",
      name: "Note",
      type: "note",
      x: 100,
      y: 100,
      width: 100,
      height: 100,
      ports: [],
    });

    expect(hitTest(doc, { x: 100, y: 150 })).toEqual({ kind: "node", id: "note" });
  });

  it("keeps notes unconnectable when a legacy document omits ports", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = addNode(doc, {
      id: "legacy-note",
      name: "Legacy note",
      type: "note",
      x: 100,
      y: 100,
      width: 100,
      height: 100,
    });

    expect(hitTest(doc, { x: 100, y: 150 })).toEqual({
      kind: "node",
      id: "legacy-note",
    });
  });

  it("keeps legacy non-empty ports connectable without using their IDs", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = addNode(doc, {
      id: "legacy",
      name: "Legacy",
      type: "service",
      x: 100,
      y: 100,
      width: 100,
      height: 100,
      ports: [{ id: "in", direction: "in", side: "left" }],
    });

    expect(hitTest(doc, { x: 150, y: 100 })).toMatchObject({
      kind: "port",
      nodeId: "legacy",
      portId: "top",
    });
  });

  it("detects every segment of an automatically routed relationship", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = addNode(doc, {
      id: "source",
      name: "Source",
      type: "service",
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });
    doc = addNode(doc, {
      id: "target",
      name: "Target",
      type: "service",
      x: 100,
      y: 300,
      width: 100,
      height: 100,
    });
    doc = addRelationship(doc, {
      sourceNodeId: "source",
      targetNodeId: "target",
      type: "calls",
    });

    expect(hitTest(doc, { x: 100, y: 200 }, { showPorts: false })).toMatchObject({
      kind: "relationship",
    });
  });
});
