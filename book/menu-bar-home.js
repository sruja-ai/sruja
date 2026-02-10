/**
 * Add a home icon to the mdBook top menu bar; click navigates to book index.
 * Injected as the first item in .left-buttons so it appears at the far left.
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

  var homeHref = pathToRoot() + "index.html";
  var a = document.createElement("a");
  a.href = homeHref;
  a.className = "sruja-nav-home icon-button";
  a.setAttribute("title", "Home");
  a.setAttribute("aria-label", "Go to home");
  a.innerHTML =
    '<span class="fa-svg"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 576 512" aria-hidden="true"><path fill="currentColor" d="M575.8 255.5c0 18-15 32.1-32 32.1h-32l.7 160.2c0 2.7-.2 5.4-.5 8.1V472c0 22.1-17.9 40-40 40H456c-1.1 0-2.2 0-3.3-.1c-1.4 .1-2.8 .1-4.2 .1H416 392c-22.1 0-40-17.9-40-40V360c0-.9 0-1.9 .1-2.8V287.6H32c-18 0-32-14.1-32-32.1c0-9 3-17 10-24L266.4 8c7-7 15-8 22-8s15 2 21 7L564.8 231.5c8 7 12 15 11 24z"/></svg></span>';
  leftButtons.insertBefore(a, leftButtons.firstChild);
})();
