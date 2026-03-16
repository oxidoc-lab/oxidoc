(function () {
  var toggle = document.querySelector(".oxidoc-menu-toggle");
  var sidebar = document.querySelector(".oxidoc-sidebar");
  var overlay = document.querySelector(".oxidoc-sidebar-overlay");
  if (!toggle || !sidebar) return;

  var mainMenu = sidebar.querySelector(".oxidoc-mobile-nav-links");
  var docNav = sidebar.querySelector(".oxidoc-sidebar-doc-nav");
  var backBtn = sidebar.querySelector(".oxidoc-mobile-back-btn");

  var scrollY = 0;

  function open() {
    scrollY = window.scrollY;
    sidebar.classList.add("open");
    if (overlay) overlay.classList.add("open");
    document.body.style.overflow = "hidden";
    document.body.style.position = "fixed";
    document.body.style.top = "-" + scrollY + "px";
    document.body.style.width = "100%";
    toggle.setAttribute("aria-expanded", "true");
    // Show doc nav by default if it exists, otherwise main menu
    if (docNav) {
      showDocNav();
    } else {
      showMainMenu();
    }
  }

  function close() {
    sidebar.classList.remove("open");
    if (overlay) overlay.classList.remove("open");
    document.body.style.overflow = "";
    document.body.style.position = "";
    document.body.style.top = "";
    document.body.style.width = "";
    window.scrollTo(0, scrollY);
    toggle.setAttribute("aria-expanded", "false");
  }

  function showMainMenu() {
    if (mainMenu) mainMenu.classList.add("active");
    if (docNav) docNav.classList.remove("active");
    if (backBtn) backBtn.style.display = "none";
  }

  function showDocNav() {
    if (mainMenu) mainMenu.classList.remove("active");
    if (docNav) docNav.classList.add("active");
    if (backBtn) backBtn.style.display = "";
  }

  toggle.addEventListener("click", function () {
    if (sidebar.classList.contains("open")) {
      close();
    } else {
      open();
    }
  });

  if (overlay) {
    overlay.addEventListener("click", close);
  }

  if (backBtn) {
    backBtn.addEventListener("click", function () {
      showMainMenu();
    });
  }

  // Main menu links that point to sections with doc nav should switch to doc nav
  if (mainMenu) {
    mainMenu.querySelectorAll("a").forEach(function (link) {
      link.addEventListener("click", function () {
        close();
      });
    });
  }

  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && sidebar.classList.contains("open")) {
      close();
    }
  });

  // Close when clicking a doc nav link
  if (docNav) {
    docNav.querySelectorAll("a").forEach(function (link) {
      link.addEventListener("click", close);
    });
  }

  // TOC dropdown toggle
  var tocToggle = document.querySelector(".oxidoc-toc-mobile-toggle");
  var tocDropdown = document.querySelector(".oxidoc-toc-mobile-dropdown");
  if (tocToggle && tocDropdown) {
    tocToggle.addEventListener("click", function () {
      var expanded = tocToggle.getAttribute("aria-expanded") === "true";
      tocToggle.setAttribute("aria-expanded", String(!expanded));
      tocDropdown.classList.toggle("open");
    });
    tocDropdown.querySelectorAll("a").forEach(function (link) {
      link.addEventListener("click", function () {
        tocToggle.setAttribute("aria-expanded", "false");
        tocDropdown.classList.remove("open");
      });
    });
  }
})();
