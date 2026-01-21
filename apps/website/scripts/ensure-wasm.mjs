// Website WASM build entrypoint.
//
// This branch is Rust-first: we build the Rust WASM bundle (wasm-bindgen/wasm-pack)
// into `apps/website/public/wasm/rust/`.
//
// We keep this wrapper file so existing npm scripts remain unchanged.
import "./ensure-wasm-rust.mjs";
