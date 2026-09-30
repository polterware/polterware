import { beforeEach, describe, expect, it } from "vitest";

import {
  detailPanelDefaultState,
  detailPanelStore,
  type DetailTarget,
} from "./index";

const scopeId = "project-alpha";
const firstTarget: DetailTarget = { scopeId, entityId: "service-api", entityKind: "service" };
const secondTarget: DetailTarget = { scopeId, entityId: "worker-sync", entityKind: "process" };

function resetStore() {
  detailPanelStore.setState(detailPanelDefaultState);
}

describe("detailPanelStore", () => {
  beforeEach(resetStore);

  it("starts with the documented layout and session defaults", () => {
    expect(detailPanelStore.getState()).toMatchObject(detailPanelDefaultState);
  });

  it("opens the panel when a new scope registers with ready targets", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: scopeId,
      activeTarget: firstTarget,
      collectionStatus: "ready",
      mode: "open",
      restoreMode: "open",
    });
  });

  it("preserves a collapsed panel when the tab scope changes", () => {
    const nextScopeId = "project-beta";
    const nextTarget: DetailTarget = { scopeId: nextScopeId, entityId: "database-primary", entityKind: "resource" };
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().collapse();

    detailPanelStore.getState().unregisterCollection(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId: nextScopeId, status: "ready", targets: [nextTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: nextScopeId,
      activeTarget: nextTarget,
      mode: "collapsed",
      restoreMode: "collapsed",
    });
  });

  it("preserves an open panel when the tab scope changes", () => {
    const nextScopeId = "project-beta";
    const nextTarget: DetailTarget = { scopeId: nextScopeId, entityId: "database-primary", entityKind: "resource" };
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });

    detailPanelStore.getState().unregisterCollection(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId: nextScopeId, status: "ready", targets: [nextTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: nextScopeId,
      activeTarget: nextTarget,
      mode: "open",
      restoreMode: "open",
    });
  });

  it("exits maximized to open on scope change when maximized from open", () => {
    const nextScopeId = "project-beta";
    const nextTarget: DetailTarget = { scopeId: nextScopeId, entityId: "database-primary", entityKind: "resource" };
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().maximize();

    detailPanelStore.getState().unregisterCollection(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId: nextScopeId, status: "ready", targets: [nextTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: nextScopeId,
      activeTarget: nextTarget,
      mode: "open",
      restoreMode: "open",
    });
  });

  it("exits maximized to collapsed on scope change when maximized from collapsed", () => {
    const nextScopeId = "project-beta";
    const nextTarget: DetailTarget = { scopeId: nextScopeId, entityId: "database-primary", entityKind: "resource" };
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().collapse();
    detailPanelStore.getState().maximize();

    detailPanelStore.getState().unregisterCollection(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId: nextScopeId, status: "ready", targets: [nextTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: nextScopeId,
      activeTarget: nextTarget,
      mode: "collapsed",
      restoreMode: "collapsed",
    });
  });

  it("consumes a matching pending target when its collection is ready", () => {
    detailPanelStore.getState().requestTarget(secondTarget);
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });

    expect(detailPanelStore.getState()).toMatchObject({ activeTarget: secondTarget, pendingTarget: null, mode: "open" });
  });

  it("drops a missing pending target and activates the first available row", () => {
    detailPanelStore.getState().requestTarget({
      scopeId,
      entityId: "missing-service",
      entityKind: "service",
    });
    detailPanelStore.getState().registerCollection({
      scopeId,
      status: "ready",
      targets: [firstTarget, secondTarget],
    });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: firstTarget,
      pendingTarget: null,
      mode: "open",
    });
  });

  it("keeps a cross-scope pending target while the previous collection unregisters", () => {
    const nextTarget: DetailTarget = {
      scopeId: "project-beta",
      entityId: "database-primary",
      entityKind: "resource",
    };
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().requestTarget(nextTarget);

    detailPanelStore.getState().unregisterCollection(scopeId);

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: null,
      activeTarget: null,
      pendingTarget: nextTarget,
      collectionStatus: "idle",
      mode: "open",
    });
  });

  it("preserves a valid active target when a collection refreshes", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });
    detailPanelStore.getState().activateTarget(secondTarget);
    detailPanelStore.getState().collapse();
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });

    expect(detailPanelStore.getState()).toMatchObject({ activeTarget: secondTarget, mode: "collapsed" });
  });

  it("opens a collapsed panel when activating another target", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });
    detailPanelStore.getState().collapse();
    detailPanelStore.getState().activateTarget(secondTarget);

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: secondTarget,
      mode: "open",
      restoreMode: "open",
    });
  });

  it("keeps maximized when activating another target", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });
    detailPanelStore.getState().maximize();
    detailPanelStore.getState().activateTarget(secondTarget);

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: secondTarget,
      mode: "maximized",
    });
  });

  it("clears and collapses an empty ready collection", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [] });

    expect(detailPanelStore.getState()).toMatchObject({ activeTarget: null, collectionStatus: "ready", mode: "collapsed" });
  });

  it("dismissTarget dismisses the active entity and keeps the empty selection on refresh", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });
    detailPanelStore.getState().dismissTarget(scopeId);

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      outgoingTarget: firstTarget,
      presence: "dismissing",
      mode: "collapsed",
    });

    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      outgoingTarget: firstTarget,
      presence: "dismissing",
      mode: "collapsed",
    });
  });

  it("keeps an in-flight or completed dismissal through same-scope loading refreshes", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().dismissTarget(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId, status: "loading", targets: [] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      outgoingTarget: firstTarget,
      collectionStatus: "loading",
      presence: "dismissing",
      mode: "collapsed",
    });

    detailPanelStore.getState().completeDismissal(scopeId);
    detailPanelStore.getState().registerCollection({ scopeId, status: "loading", targets: [] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      outgoingTarget: null,
      collectionStatus: "loading",
      presence: "hidden",
      mode: "collapsed",
    });
  });

  it("dismissTarget ignores a different scope", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().dismissTarget("other-scope");

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: firstTarget,
      mode: "open",
    });
  });

  it("completeDismissal transitions from dismissing to hidden", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().dismissTarget(scopeId);

    expect(detailPanelStore.getState().presence).toBe("dismissing");
    expect(detailPanelStore.getState().outgoingTarget).toBe(firstTarget);

    detailPanelStore.getState().completeDismissal(scopeId);

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      outgoingTarget: null,
      presence: "hidden",
      mode: "collapsed",
    });
  });

  it("cancels dismissal when another target becomes active", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget, secondTarget] });
    detailPanelStore.getState().dismissTarget(scopeId);
    detailPanelStore.getState().activateTarget(secondTarget);

    detailPanelStore.getState().completeDismissal(scopeId);

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: secondTarget,
      outgoingTarget: null,
      presence: "visible",
      mode: "open",
    });
  });

  it("keeps the panel hidden while the collection is loading on first open", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "loading", targets: [] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      collectionStatus: "loading",
      presence: "hidden",
    });
  });

  // KNOWN BUG (commit 6485f0a): a cold load leaves activeTarget null and presence hidden, which
  // the ready branch reads as an explicit empty selection, so the panel never reveals itself.
  // it.fails flips to a failure once fixed; then change it back to it(...).
  it.fails("selects the first target when loading in the current scope becomes ready", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "loading", targets: [] });
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: firstTarget,
      outgoingTarget: null,
      presence: "visible",
      mode: "open",
    });
  });

  it("keeps the panel visible through same-scope loading when it was already open", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    expect(detailPanelStore.getState().presence).toBe("visible");

    detailPanelStore.getState().registerCollection({ scopeId, status: "loading", targets: [] });

    expect(detailPanelStore.getState()).toMatchObject({
      activeTarget: null,
      collectionStatus: "loading",
      presence: "visible",
    });
  });

  it("clears the current collection session state when it unregisters", () => {
    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().unregisterCollection(scopeId);

    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: null,
      activeTarget: null,
      pendingTarget: null,
      collectionStatus: "idle",
      mode: "open",
      restoreMode: "open",
    });
  });

  it("tracks per-collection placement and resets to bottom on unregister", () => {
    expect(detailPanelStore.getState().placement).toBe("bottom");

    detailPanelStore.getState().registerCollection({
      scopeId,
      status: "ready",
      targets: [firstTarget],
      placement: "side",
    });
    expect(detailPanelStore.getState().placement).toBe("side");

    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    expect(detailPanelStore.getState().placement).toBe("bottom");

    detailPanelStore.getState().registerCollection({
      scopeId,
      status: "ready",
      targets: [firstTarget],
      placement: "side",
    });
    detailPanelStore.getState().unregisterCollection(scopeId);
    expect(detailPanelStore.getState().placement).toBe("bottom");
  });

  it("maximizes only an active collection and restores to the recorded restoreMode", () => {
    detailPanelStore.getState().maximize();
    expect(detailPanelStore.getState().mode).toBe("open");

    detailPanelStore.getState().registerCollection({ scopeId, status: "ready", targets: [firstTarget] });
    detailPanelStore.getState().collapse();
    detailPanelStore.getState().maximize();

    expect(detailPanelStore.getState()).toMatchObject({ mode: "maximized", restoreMode: "collapsed" });

    detailPanelStore.getState().restore();
    expect(detailPanelStore.getState()).toMatchObject({ mode: "collapsed", restoreMode: "collapsed" });
  });

  it("hides a stale foreign selection when an empty collection registers", () => {
    const canvasTarget: DetailTarget = { scopeId: "architecture:canvas", entityId: "node-a", entityKind: "arch-node" };
    detailPanelStore.getState().registerCollection({ scopeId: "architecture:canvas", status: "ready", targets: [canvasTarget] });
    detailPanelStore.getState().unregisterCollection("architecture:map");
    expect(detailPanelStore.getState().presence).toBe("visible");

    detailPanelStore.getState().registerCollection({ scopeId: "inbox:items", status: "ready", targets: [] });
    expect(detailPanelStore.getState()).toMatchObject({
      activeScope: "inbox:items",
      activeTarget: null,
      mode: "collapsed",
      presence: "hidden",
    });

    detailPanelStore.getState().unregisterCollection("inbox:items");
    expect(detailPanelStore.getState()).toMatchObject({ activeTarget: null, presence: "hidden" });
  });

  it("keeps snap geometry out of global and persisted state", () => {
    expect(detailPanelStore.getState()).not.toHaveProperty("bottomSize");
    expect(detailPanelStore.getState()).not.toHaveProperty("setSize");
  });
});
