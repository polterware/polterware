/** Kebab-case card slug (the Polter `## slug: Title` handle). */
export function cardSlugFromTitle(title: string): string {
  const slug = title
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48)
    .replace(/-+$/g, "");
  return slug || "card";
}

export function uniqueCardSlug(existingSlugs: readonly string[], title: string): string {
  const base = cardSlugFromTitle(title);
  const used = new Set(existingSlugs);
  if (!used.has(base)) return base;
  let suffix = 2;
  while (used.has(`${base}-${suffix}`)) suffix += 1;
  return `${base}-${suffix}`;
}
