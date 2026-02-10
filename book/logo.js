/**
 * Inject Sruja logo into mdBook sidebar (runs on every page).
 * Assumes sruja-logo.png is at the book output root (copied by copy-wasm.sh).
 */
(function () {
  function pathToRoot() {
    var path = window.location.pathname;
    // Remove leading slash and trailing filename; count depth
    var parts = path.replace(/^\//, "").split("/").filter(Boolean);
    var depth = Math.max(0, parts.length - 1);
    return depth === 0 ? "" : Array(depth).fill("..").join("/") + "/";
  }
  var sidebar = document.querySelector(".sidebar");
  if (!sidebar) return;
  var prefix = pathToRoot();
  var img = document.createElement("img");
  img.src = prefix + "sruja-logo.png";
  img.alt = "Sruja";
  img.className = "sruja-sidebar-logo";
  img.setAttribute("title", "Sruja – Architecture as code");
  sidebar.insertBefore(img, sidebar.firstChild);
})();
