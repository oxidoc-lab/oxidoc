(function () {
  var btn = document.createElement("button");
  btn.className = "oxidoc-back-to-top";
  btn.setAttribute("aria-label", "Back to top");
  btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="m11 7.825l-4.9 4.9q-.3.3-.7.288t-.7-.313q-.275-.3-.288-.7t.288-.7l6.6-6.6q.15-.15.325-.212T12 4.425t.375.063t.325.212l6.6 6.6q.275.275.275.688t-.275.712q-.3.3-.712.3t-.713-.3L13 7.825V19q0 .425-.288.713T12 20t-.712-.288T11 19z"/></svg>';
  document.body.appendChild(btn);

  var visible = false;
  function toggle() {
    var show = window.scrollY > 300;
    if (show !== visible) {
      visible = show;
      btn.classList.toggle("visible", show);
    }
  }

  var ticking = false;
  window.addEventListener("scroll", function () {
    if (!ticking) {
      requestAnimationFrame(function () {
        toggle();
        ticking = false;
      });
      ticking = true;
    }
  });

  btn.addEventListener("click", function () {
    window.scrollTo({ top: 0, behavior: "smooth" });
  });
})();
