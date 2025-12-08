import { User, Database, Server, Inbox, Home } from "lucide";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

type IconNode = [string, Record<string, string>];
type Icon = IconNode[];

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const toMarkup = (icon: Icon): string =>
  icon
    .map(([tag, attrs]) =>
      `<${tag} ${Object.entries(attrs)
        .map(([k, v]) => `${k}="${v}"`)
        .join(" ")} />`
    )
    .join("");

const icons = {
  user: toMarkup(User as unknown as Icon),
  database: toMarkup(Database as unknown as Icon),
  server: toMarkup(Server as unknown as Icon),
  inbox: toMarkup(Inbox as unknown as Icon),
  home: toMarkup(Home as unknown as Icon),
};

const outPath = path.resolve(
  __dirname,
  "../../../../pkg/export/svg/icons/icons.json"
);
fs.writeFileSync(outPath, JSON.stringify(icons, null, 2));
