/**
 * The comparison with the GPUI Desktop (`tooling/parity`): with `tour=1` on the renderer URL
 * (Main sets it under POLTERWARE_TOUR), the window walks the stops `desktop-gpui/src/shots.rs`
 * walks, under the same names, so each pair of captures shows the same screen over the same
 * data. With `bench=1` (POLTERWARE_BENCH) it reports when the Projects screen first shows rows.
 */

/** What the tour drives; App supplies it from its own navigation. */
export type TourDriver = {
  /** The project list has loaded. */
  ready(): boolean;
  projects(): { id: string; name: string }[];
  home(): void;
  enterProject(id: string): void;
  section(id: string): void;
  projectSettings(open: boolean): void;
  commander(open: boolean): void;
  appSettings(open: boolean): void;
};

/** The project the tour opens: the seed's main one (tooling/bench/seed.mjs), else the first. */
const TOUR_PROJECT = "Atlas";

/** Knowledge's tabs in order: the stop for the tab's list (none for Decisions, the section's own stop) and for its first record's page. */
const KNOWLEDGE_STOPS: [string, string | null, string | null][] = [
  ["Decisions", null, "07-knowledge-decision"],
  ["Rules", "07-knowledge-rules", "07-knowledge-rule"],
  ["Docs", "07-knowledge-docs", "07-knowledge-doc"],
];

/** Sidebar sections after the Inbox, with their stop names (route ids are the app's). */
const SECTIONS: [string, string][] = [
  ["03-activity", "activity"],
  ["04-milestones", "milestones"],
  ["05-canvas", "canvas"],
  ["06-structure", "structure"],
  ["07-knowledge", "truth"],
];

/** The open Inbox item's actions the tour opens, with their stop names and button labels. */
const INBOX_ACTIONS: [string, string][] = [
  ["02-inbox-decide", "Decide"],
  ["02-inbox-later", "Later"],
];

/** The detail footer's action button with this label. */
const inboxAction = (label: string) =>
  [...document.querySelectorAll<HTMLButtonElement>('[aria-label="Inbox actions"] button[aria-expanded]')].find(
    (button) => button.textContent?.trim() === label,
  );

/** The first element matching `selector`, or the first whose text is `text` among them. */
const find = (selector: string, text?: string) =>
  [...document.querySelectorAll<HTMLElement>(selector)].find((element) => text === undefined || element.textContent?.trim() === text);

/** Knowledge's tab named `label`: a tab stacks a hidden copy of its label, so match the visible one. */
const knowledgeTab = (label: string) =>
  find('[role="tab"] .pw-tab__label-text', label)?.closest<HTMLElement>('[role="tab"]');

/** Escape, as a key press closes the open popover, dialog, or panel. */
const escape = () =>
  (document.activeElement ?? document.body).dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

const params = () => new URLSearchParams(window.location.search);
export const tourRequested = () => params().get("tour") === "1";
export const benchRequested = () => params().get("bench") === "1";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
/** Lets data land and motion finish, then waits for two frames to be painted. */
async function settle(ms = 1200) {
  await wait(ms);
  await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

export async function runTour(driver: TourDriver) {
  const shot = (name: string) => window.polterware.instrument.shot(name);
  for (let i = 0; i < 150 && !driver.ready(); i++) await wait(100);
  await settle(2500);
  driver.home();
  await settle();
  await shot("01-projects");
  find('[aria-label="New project"]')?.click();
  await settle();
  await shot("01-projects-new");
  escape();
  await settle(600);
  const projects = driver.projects();
  const project = projects.find((item) => item.name === TOUR_PROJECT) ?? projects[0];
  if (project) {
    driver.enterProject(project.id);
    driver.section("inbox");
    await settle(1500);
    await shot("02-inbox");
    // The detail stays closed until a row is opened: the first one, as a click would.
    find("tr.pw-data-table-row--interactive")?.click();
    await settle(600);
    // The open item's Decide and Later, each opened as a click would and closed with Cancel.
    for (const [name, label] of INBOX_ACTIONS) {
      inboxAction(label)?.click();
      await settle();
      await shot(name);
      document.querySelector<HTMLButtonElement>(".pw-morph-cancel")?.click();
      await settle(600);
    }
    // Switch project from the project's crumb, then Help and its first topic.
    find(".pw-titlebar-crumbs__project")?.click();
    await settle();
    await shot("02-switch-project");
    escape();
    await settle(600);
    find('[aria-label="Help and about"]')?.click();
    await settle();
    await shot("02-help");
    find('[role="menuitem"], button', "What it's for")?.click();
    await settle();
    await shot("02-help-topic");
    escape();
    await settle(600);
    for (const [name, section] of SECTIONS) {
      driver.section(section);
      await settle();
      if (section === "truth") {
        // Knowledge reopens on the tab it was left on, even in an earlier run: start on Decisions.
        knowledgeTab("Decisions")?.click();
        await settle();
      }
      await shot(name);
      if (section === "activity") {
        // The newest change, then the newest Dispatch entry, each opened as a click on its
        // row would.
        find('tr[aria-label^="Open change"]:not([data-row-id^="dispatch:"])')?.click();
        await settle();
        await shot("03-activity-detail");
        escape();
        await settle(600);
        find('tr[data-row-id^="dispatch:"]')?.click();
        await settle();
        await shot("03-activity-dispatch");
        escape();
        await settle(600);
      }
      if (section === "truth") {
        // Knowledge is on Decisions: the first one's page, then Rules and the first rule in
        // its Composer, then Docs with its Add doc Composer and its archive, and the first
        // doc's page, each opened as a click would and left with Back (a rule with Escape).
        for (const [tab, list, page] of KNOWLEDGE_STOPS) {
          if (list) {
            knowledgeTab(tab)?.click();
            await settle();
            await shot(list);
          }
          if (tab === "Docs") {
            find('[aria-label="Add doc"]')?.click();
            await settle();
            await shot("07-knowledge-docs-new");
            escape();
            await settle(600);
            find('[aria-label="Archived docs"]')?.click();
            await settle();
            await shot("07-knowledge-docs-archived");
            find('[aria-label="Back to docs"]')?.click();
            await settle(600);
          }
          if (page) {
            find("tr.pw-data-table-row--interactive")?.click();
            await settle();
            await shot(page);
            if (tab === "Rules") escape();
            else find('[aria-label="Back"]')?.click();
            await settle(600);
          }
        }
      }
    }
    driver.section("inbox");
    driver.projectSettings(true);
    await settle();
    await shot("08-project-settings");
    driver.projectSettings(false);
    await settle(400);
  }
  driver.commander(true);
  await settle();
  await shot("09-commander");
  driver.commander(false);
  await settle(400);
  driver.appSettings(true);
  await settle();
  await shot("10-app-settings");
  driver.appSettings(false);
  await window.polterware.instrument.done();
}
