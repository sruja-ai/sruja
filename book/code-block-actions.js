/**
 * mdBook: add "Show diagram" / "Preview" button on code blocks.
 * - language-mermaid: button renders Mermaid diagram below the block.
 * - language-sruja: button runs WASM (dslToMermaid), then renders Mermaid; optional "Copy Mermaid" / "Copy Markdown".
 * Requires: Mermaid loaded (e.g. mdbook-mermaid). For Sruja, WASM at (path_to_root + "wasm/rust/") or SRUJA_WASM_BASE.
 */
(function () {
  "use strict";

  var wasmModule = null;
  var wasmInitPromise = null;

  function getWasmBase() {
    if (typeof window.SRUJA_WASM_BASE !== "undefined") return window.SRUJA_WASM_BASE;
    var root = typeof path_to_root !== "undefined" ? path_to_root : "";
    var relative = (root || "").replace(/\/?$/, "/") + "wasm/rust/";
    try {
      return new URL(relative, window.location.href).href.replace(/\/?$/, "/");
    } catch (e) {
      return relative;
    }
  }

  function fetchWithRetry(url, maxAttempts) {
    maxAttempts = maxAttempts || 5;
    var attempt = 0;
    function doFetch() {
      return fetch(url).then(function (r) {
        if (r.ok) return r;
        if (attempt < maxAttempts - 1) {
          attempt++;
          return new Promise(function (resolve) { setTimeout(resolve, 1000); }).then(doFetch);
        }
        throw new Error("WASM JS not found: " + url);
      });
    }
    return doFetch();
  }

  function loadWasm() {
    if (wasmModule) return Promise.resolve(wasmModule);
    if (wasmInitPromise) return wasmInitPromise;
    var base = getWasmBase();
    var jsUrl = base + "sruja_wasm.js";
    var wasmUrl = base + "sruja_wasm_bg.wasm";
    wasmInitPromise = fetchWithRetry(jsUrl)
      .then(function (r) { return r.text(); })
      .then(function (code) {
        var blob = new Blob([code], { type: "application/javascript" });
        var blobUrl = URL.createObjectURL(blob);
        return import(blobUrl).then(
          function (mod) {
            URL.revokeObjectURL(blobUrl);
            var init = mod.default({ module_or_path: wasmUrl });
            return (init && typeof init.then === "function" ? init : Promise.resolve()).then(function () {
              if (typeof mod.init_panic_hook === "function") mod.init_panic_hook();
              wasmModule = mod;
              return mod;
            });
          },
          function (e) {
            URL.revokeObjectURL(blobUrl);
            throw e;
          }
        );
      });
    return wasmInitPromise;
  }

  function normalizeDsl(s) {
    return (s || "")
      .replace(/\u2192/g, "->")
      .replace(/[""]/g, '"')
      .replace(/[']/g, "'")
      .replace(/\u2013|\u2014/g, "-")
      .split(/\r?\n/)
      .map(function (line) { return line.replace(/^\s*\d+\s*[→:.-]\s?/, ""); })
      .join("\n")
      .trim();
  }

  function renderMermaidInto(container, mermaidCode, callback) {
    if (!window.mermaid) {
      if (callback) callback(new Error("Mermaid not loaded"));
      return;
    }
    var id = "mermaid-" + Math.random().toString(36).slice(2);
    var div = document.createElement("div");
    div.className = "mermaid";
    div.id = id;
    div.textContent = mermaidCode || "";
    container.innerHTML = "";
    container.appendChild(div);
    window.mermaid
      .run({ nodes: [div], suppressErrors: false })
      .then(function () {
        if (callback) callback(null);
      })
      .catch(function (err) {
        if (callback) callback(err);
      });
  }

  function addButtonStyles(btn) {
    btn.style.cssText =
      "padding:6px 10px;border-radius:6px;border:1px solid var(--table-border-color);" +
      "background:var(--bg);color:var(--fg);cursor:pointer;font-size:12px;margin-left:6px;";
  }

  function addToolbarMermaid(pre, codeEl) {
    if (pre.dataset.srujaActions === "1") return;
    pre.dataset.srujaActions = "1";
    var code = (codeEl.textContent || "").trim();
    if (!code) return;

    var wrapper = document.createElement("div");
    wrapper.className = "code-block-wrapper";
    wrapper.style.position = "relative";
    pre.parentNode.insertBefore(wrapper, pre);
    wrapper.appendChild(pre);

    var toolbar = document.createElement("div");
    toolbar.style.cssText = "position:absolute;top:8px;right:8px;z-index:10;display:flex;gap:6px;";
    var btn = document.createElement("button");
    btn.textContent = "Show diagram";
    addButtonStyles(btn);
    toolbar.appendChild(btn);
    wrapper.appendChild(toolbar);

    var preview = document.createElement("div");
    preview.style.cssText =
      "margin-top:12px;padding:12px;border:1px solid var(--table-border-color);border-radius:8px;" +
      "background:var(--quote-bg);overflow:auto;display:none;";
    wrapper.appendChild(preview);

    btn.onclick = function () {
      if (preview.style.display === "none") {
        preview.style.display = "block";
        preview.innerHTML = "<p>Rendering…</p>";
        renderMermaidInto(preview, code, function (err) {
          if (err) {
            preview.innerHTML = "<p style='color:var(--blockquote-caution-color)'>" + (err.message || String(err)) + "</p>";
          }
        });
        btn.textContent = "Hide diagram";
      } else {
        preview.style.display = "none";
        btn.textContent = "Show diagram";
      }
    };
  }

  function addToolbarSruja(pre, codeEl) {
    if (pre.dataset.srujaActions === "1") return;
    pre.dataset.srujaActions = "1";
    var dsl = (codeEl.textContent || "").trim();
    if (!dsl) return;

    var wrapper = document.createElement("div");
    wrapper.className = "code-block-wrapper";
    wrapper.style.position = "relative";
    pre.parentNode.insertBefore(wrapper, pre);
    wrapper.appendChild(pre);

    var toolbar = document.createElement("div");
    toolbar.style.cssText = "position:absolute;top:8px;right:8px;z-index:10;display:flex;gap:6px;flex-wrap:wrap;";
    var showBtn = document.createElement("button");
    showBtn.textContent = "Show diagram";
    addButtonStyles(showBtn);
    var copyMermaidBtn = document.createElement("button");
    copyMermaidBtn.textContent = "Copy Mermaid";
    addButtonStyles(copyMermaidBtn);
    var copyMdBtn = document.createElement("button");
    copyMdBtn.textContent = "Copy Markdown";
    addButtonStyles(copyMdBtn);
    toolbar.appendChild(showBtn);
    toolbar.appendChild(copyMermaidBtn);
    toolbar.appendChild(copyMdBtn);
    wrapper.appendChild(toolbar);

    var preview = document.createElement("div");
    preview.style.cssText =
      "margin-top:12px;padding:12px;border:1px solid var(--table-border-color);border-radius:8px;" +
      "background:var(--quote-bg);overflow:auto;display:none;";
    wrapper.appendChild(preview);

    var lastMermaid = null;
    var lastMarkdown = null;

    function runWasm(getOutput) {
      return loadWasm().then(
        function (mod) {
          var input = normalizeDsl(dsl);
          if (getOutput === "mermaid") {
            var out = mod.sruja_dsl_to_mermaid(input);
            lastMermaid = out;
            return out;
          }
          if (getOutput === "markdown") {
            var md = mod.sruja_dsl_to_markdown(input);
            lastMarkdown = md;
            return md;
          }
          return null;
        },
        function (err) {
          return Promise.reject(err);
        }
      );
    }

    showBtn.onclick = function () {
      if (preview.style.display === "none") {
        preview.style.display = "block";
        preview.innerHTML = "<p>Loading WASM and rendering…</p>";
        runWasm("mermaid")
          .then(function (mermaidCode) {
            preview.innerHTML = "";
            renderMermaidInto(preview, mermaidCode, function (err) {
              if (err) {
                preview.innerHTML = "<p style='color:var(--blockquote-caution-color)'>" + (err.message || String(err)) + "</p>";
              }
            });
          })
          .catch(function (err) {
            var msg = (err.message || String(err));
            preview.innerHTML =
              "<p style='color:var(--blockquote-caution-color)'>WASM not available: " + msg + ".</p>" +
              "<p><strong>Fix:</strong> From the repo root run <code>make wasm</code>, then <code>make book-serve</code>. If you just started serve, wait a few seconds and click <strong>Retry</strong>.</p>" +
              "<button type='button' class='sruja-retry-btn' style='padding:6px 12px;border-radius:6px;border:1px solid var(--table-border-color);background:var(--bg);color:var(--fg);cursor:pointer;'>Retry</button>";
            var retryBtn = preview.querySelector(".sruja-retry-btn");
            if (retryBtn) {
              retryBtn.onclick = function () {
                preview.innerHTML = "<p>Loading WASM…</p>";
                runWasm("mermaid").then(function (mermaidCode) {
                  preview.innerHTML = "";
                  renderMermaidInto(preview, mermaidCode, function (e) {
                    if (e) preview.innerHTML = "<p style='color:var(--blockquote-caution-color)'>" + (e.message || String(e)) + "</p>";
                  });
                }).catch(function (e) {
                  preview.innerHTML = "<p style='color:var(--blockquote-caution-color)'>WASM not available: " + (e.message || String(e)) + ". <button type='button' class='sruja-retry-btn' style='padding:6px 12px;margin-left:8px;'>Retry</button></p>";
                  var b = preview.querySelector(".sruja-retry-btn");
                  if (b) b.onclick = retryBtn.onclick;
                });
              };
            }
          });
        showBtn.textContent = "Hide diagram";
      } else {
        preview.style.display = "none";
        showBtn.textContent = "Show diagram";
      }
    };

    copyMermaidBtn.onclick = function () {
      (lastMermaid ? Promise.resolve(lastMermaid) : runWasm("mermaid"))
        .then(function (text) {
          return navigator.clipboard.writeText(text || "").then(function () {
            copyMermaidBtn.textContent = "Copied!";
            setTimeout(function () { copyMermaidBtn.textContent = "Copy Mermaid"; }, 1500);
          });
        })
        .catch(function (err) {
          alert("Failed: " + (err.message || String(err)));
        });
    };

    copyMdBtn.onclick = function () {
      (lastMarkdown ? Promise.resolve(lastMarkdown) : runWasm("markdown"))
        .then(function (text) {
          return navigator.clipboard.writeText(text || "").then(function () {
            copyMdBtn.textContent = "Copied!";
            setTimeout(function () { copyMdBtn.textContent = "Copy Markdown"; }, 1500);
          });
        })
        .catch(function (err) {
          alert("Failed: " + (err.message || String(err)));
        });
    };
  }

  function run() {
    if (!window.mermaid) {
      setTimeout(run, 100);
      return;
    }
    var pres = document.querySelectorAll("pre code");
    for (var i = 0; i < pres.length; i++) {
      var codeEl = pres[i];
      var pre = codeEl.parentElement;
      if (!pre || pre.dataset.srujaActions === "1") continue;
      var cls = (codeEl.className || "") + " " + (pre.className || "");
      if (/language-mermaid/.test(cls)) {
        addToolbarMermaid(pre, codeEl);
      } else if (/language-sruja/.test(cls)) {
        addToolbarSruja(pre, codeEl);
      }
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run);
  } else {
    run();
  }
})();
