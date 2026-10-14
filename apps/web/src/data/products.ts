/** The command the site shows for the Lore CLI. One place to change if the binary is renamed. */
export const CLI = "lore";

export const REPO_URL = "https://github.com/polterware/polterware";
export const RELEASES_URL = `${REPO_URL}/releases`;
export const INSTALL_SCRIPT = "https://raw.githubusercontent.com/polterware/polterware/main/scripts/install-desktop.sh";

export const installCommand = (app?: "lore" | "vitals") =>
  `curl -fsSL ${INSTALL_SCRIPT} | ${app === "lore" ? "APP=lore " : ""}bash`;

export type ProductId = "lore" | "cli" | "vitals";

export type Product = {
  id: ProductId;
  name: string;
  href: string;
  /** One line for menus and cards. */
  summary: string;
  status: string;
};

export const products: Record<ProductId, Product> = {
  lore: {
    id: "lore",
    name: "Lore",
    href: "/lore",
    summary: "Shared, evidence-backed project context for people and coding agents.",
    status: "Alpha · macOS on Apple Silicon",
  },
  cli: {
    id: "cli",
    name: "Lore CLI",
    href: "/lore/cli",
    summary: "Lore from your terminal: a TUI, scripted reads and writes, and an MCP bridge.",
    status: "Bundled with Lore · builds from source",
  },
  vitals: {
    id: "vitals",
    name: "Vitals",
    href: "/vitals",
    summary: "Watch remote machines over SSH: metrics, processes and a terminal.",
    status: "Alpha · macOS on Apple Silicon",
  },
};

export const productList: Product[] = [products.lore, products.cli, products.vitals];
