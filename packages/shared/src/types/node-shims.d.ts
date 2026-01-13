declare module "fs/promises";
declare module "path";
declare module "url";

// These are type declarations for Node.js environment compatibility
// eslint-disable-next-line @typescript-eslint/no-unused-vars
declare const window: Window & typeof globalThis;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
declare const global: typeof globalThis;

export {};
