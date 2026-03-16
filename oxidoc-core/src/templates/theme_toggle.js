(function () {
  var btn = document.querySelector(".oxidoc-theme-toggle");
  if (!btn) return;

  var modes = ["system", "light", "dark"];
  var icons = {
    system: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M8.1 21.213q-1.825-.788-3.175-2.138T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22t-3.9-.788m4.9-1.287q2.975-.375 4.988-2.613T20 12t-2.013-5.312T13 4.075z"/></svg>`,
    light: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M8.463 15.538Q7 14.075 7 12t1.463-3.537T12 7t3.538 1.463T17 12t-1.463 3.538T12 17t-3.537-1.463M2 13q-.425 0-.712-.288T1 12t.288-.712T2 11h2q.425 0 .713.288T5 12t-.288.713T4 13zm18 0q-.425 0-.712-.288T19 12t.288-.712T20 11h2q.425 0 .713.288T23 12t-.288.713T22 13zm-8.712-8.287Q11 4.425 11 4V2q0-.425.288-.712T12 1t.713.288T13 2v2q0 .425-.288.713T12 5t-.712-.288m0 18Q11 22.426 11 22v-2q0-.425.288-.712T12 19t.713.288T13 20v2q0 .425-.288.713T12 23t-.712-.288M5.65 7.05L4.575 6q-.3-.275-.288-.7t.288-.725q.3-.3.725-.3t.7.3L7.05 5.65q.275.3.275.7t-.275.7t-.687.288t-.713-.288M18 19.425l-1.05-1.075q-.275-.3-.275-.712t.275-.688q.275-.3.688-.287t.712.287L19.425 18q.3.275.288.7t-.288.725q-.3.3-.725.3t-.7-.3M16.95 7.05q-.3-.275-.288-.687t.288-.713L18 4.575q.275-.3.7-.288t.725.288q.3.3.3.725t-.3.7L18.35 7.05q-.3.275-.7.275t-.7-.275M4.575 19.425q-.3-.3-.3-.725t.3-.7l1.075-1.05q.3-.275.712-.275t.688.275q.3.275.288.688t-.288.712L6 19.425q-.275.3-.7.288t-.725-.288"/></svg>`,
    dark: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M12 21q-3.775 0-6.387-2.613T3 12q0-3.45 2.25-5.988T11 3.05q.325-.05.575.088t.4.362t.163.525t-.188.575q-.425.65-.638 1.375T11.1 7.5q0 2.25 1.575 3.825T16.5 12.9q.775 0 1.538-.225t1.362-.625q.275-.175.563-.162t.512.137q.25.125.388.375t.087.6q-.35 3.45-2.937 5.725T12 21"/></svg>`
  };
  var labels = {
    system: "System theme",
    light: "Light theme",
    dark: "Dark theme"
  };

  function getStored() {
    try { return localStorage.getItem("oxidoc-theme"); } catch (e) { return null; }
  }

  function setStored(v) {
    try { localStorage.setItem("oxidoc-theme", v); } catch (e) {}
  }

  function isDark(mode) {
    if (mode === "dark") return true;
    if (mode === "light") return false;
    return window.matchMedia("(prefers-color-scheme:dark)").matches;
  }

  function apply(mode) {
    var html = document.documentElement;
    if (mode === "system") {
      html.removeAttribute("data-theme");
    } else {
      html.setAttribute("data-theme", mode);
    }
    btn.innerHTML = icons[mode];
    btn.setAttribute("aria-label", labels[mode]);
    btn.setAttribute("title", labels[mode]);
    // Re-render mermaid diagrams with correct theme
    if (typeof window.mermaid !== "undefined") {
      var theme = isDark(mode) ? "dark" : "default";
      window.mermaid.initialize({ startOnLoad: false, theme: theme });
      var els = document.querySelectorAll("pre.mermaid");
      els.forEach(function (el) {
        if (!el.getAttribute("data-mermaid-src")) {
          el.setAttribute("data-mermaid-src", el.textContent);
        }
        var src = el.getAttribute("data-mermaid-src");
        el.removeAttribute("data-processed");
        el.textContent = src;
      });
      if (els.length) window.mermaid.run();
    }
  }

  var current = getStored() || "system";
  apply(current);

  btn.addEventListener("click", function () {
    var idx = modes.indexOf(current);
    current = modes[(idx + 1) % modes.length];
    setStored(current);
    apply(current);
  });
})();
