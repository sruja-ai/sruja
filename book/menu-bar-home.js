/**
 * Add Sruja logo to the mdBook top menu bar; click navigates to book index.
 * Injected as the first item in .left-buttons. Assumes sruja-logo.png at book root (copy-wasm.sh).
 */
(function () {
  function pathToRoot() {
    var path = window.location.pathname;
    var parts = path.replace(/^\//, "").split("/").filter(Boolean);
    var depth = Math.max(0, parts.length - 1);
    return depth === 0 ? "" : Array(depth).fill("..").join("/") + "/";
  }

  var menuBar = document.getElementById("mdbook-menu-bar");
  if (!menuBar) return;

  var leftButtons = menuBar.querySelector(".left-buttons");
  if (!leftButtons) return;

  var prefix = pathToRoot();
  var a = document.createElement("a");
  a.href = prefix + "index.html";
  a.className = "sruja-nav-logo";
  a.setAttribute("title", "Sruja (beta) – Home");
  a.setAttribute("aria-label", "Go to home");
  var img = document.createElement("img");
  img.src = prefix + "sruja-logo.png";
  img.alt = "Sruja (beta)";
  img.className = "sruja-nav-logo-img";
  a.appendChild(img);
  leftButtons.insertBefore(a, leftButtons.firstChild);
})();
