(function () {
  var t = document.querySelector(".oxidoc-toc");
  if (!t) return;
  var links = t.querySelectorAll('a[href^="#"]');
  if (!links.length) return;

  var ids = [];
  links.forEach(function (a) {
    var id = a.getAttribute("href").slice(1);
    if (id) ids.push({ el: document.getElementById(id), a: a });
  });
  ids = ids.filter(function (o) {
    return o.el;
  });
  if (!ids.length) return;

  var active = null;
  function update() {
    var found = null;
    var vh = window.innerHeight;
    var docH = document.documentElement.scrollHeight;
    var scrollY = window.scrollY || window.pageYOffset;

    // At bottom of page: activate the last heading
    if (scrollY + vh >= docH - 2) {
      found = ids[ids.length - 1].a;
    } else {
      // Walk backwards: first heading in the upper 40% of viewport wins
      for (var i = ids.length - 1; i >= 0; i--) {
        if (ids[i].el.getBoundingClientRect().top <= vh * 0.4) {
          found = ids[i].a;
          break;
        }
      }
    }

    if (!found) found = ids[0] && ids[0].a;
    if (found !== active) {
      if (active) active.classList.remove("active");
      if (found) found.classList.add("active");
      active = found;
    }
  }
  window.addEventListener("scroll", update, { passive: true });
  update();
})();
