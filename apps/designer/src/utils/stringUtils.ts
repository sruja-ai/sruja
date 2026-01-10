export const slugify = (text: string): string =>
  text
    .toLowerCase()
    .trim()
    .replace(/[\u0080-\uFFFF]/g, "")
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "");
