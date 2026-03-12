(function () {
  var header = document.querySelector(".oxidoc-header");
  if (!header) return;
  var lastY = window.scrollY;
  var ticking = false;
  window.addEventListener("scroll", function () {
    if (!ticking) {
      requestAnimationFrame(function () {
        var y = window.scrollY;
        if (y > lastY && y > 60) {
          header.classList.add("oxidoc-header-hidden");
        } else {
          header.classList.remove("oxidoc-header-hidden");
        }
        lastY = y;
        ticking = false;
      });
      ticking = true;
    }
  });
})();
