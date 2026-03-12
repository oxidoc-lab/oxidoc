(function () {
  var btn = document.querySelector(".oxidoc-theme-toggle");
  if (!btn) return;

  var icon = btn.querySelector("iconify-icon");
  var modes = ["system", "light", "dark"];
  var icons = {
    system: "material-symbols:contrast",
    light: "material-symbols:light-mode-rounded",
    dark: "material-symbols:dark-mode-rounded"
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

  function apply(mode) {
    var html = document.documentElement;
    if (mode === "system") {
      html.removeAttribute("data-theme");
    } else {
      html.setAttribute("data-theme", mode);
    }
    if (icon) icon.setAttribute("icon", icons[mode]);
    btn.setAttribute("aria-label", labels[mode]);
    btn.setAttribute("title", labels[mode]);
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
