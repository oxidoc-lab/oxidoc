(function () {
  var btn = document.createElement("button");
  btn.className = "oxidoc-back-to-top";
  btn.setAttribute("aria-label", "Back to top");
  btn.innerHTML = '<iconify-icon icon="material-symbols:arrow-upward-rounded" width="20" height="20"></iconify-icon>';
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
