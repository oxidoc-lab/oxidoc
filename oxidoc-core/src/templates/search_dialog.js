(function () {
  var overlay = document.getElementById("oxidoc-search-overlay");
  var input = document.getElementById("oxidoc-search-input");
  var clearBtn = document.getElementById("oxidoc-search-clear");
  var closeBtn = document.getElementById("oxidoc-search-close");
  var body = document.getElementById("oxidoc-search-body");
  var resultsEl = document.getElementById("oxidoc-search-results");
  var emptyEl = document.getElementById("oxidoc-search-empty");
  var noResultsEl = document.getElementById("oxidoc-search-no-results");
  if (!overlay || !input) return;

  var wasmModule = null;
  var indexLoaded = false;
  var activeIdx = -1;
  var debounceTimer = null;
  var lastResultKey = "";
  var loadedChunks = {};

  // Preload search result icons so they're cached before results render
  var searchIcons = ["material-symbols:tag-rounded", "material-symbols:description-rounded"];
  searchIcons.forEach(function (name) {
    var el = document.createElement("iconify-icon");
    el.setAttribute("icon", name);
    el.style.position = "absolute";
    el.style.visibility = "hidden";
    el.style.pointerEvents = "none";
    document.body.appendChild(el);
  });

  // Load the Wasm search module and initialize with the metadata
  function ensureSearch(cb) {
    if (indexLoaded) return cb();
    // Load wasm module
    var loadWasm = window.__oxidoc_search ? window.__oxidoc_search() : Promise.reject("no loader");
    loadWasm.then(function (mod) {
      wasmModule = mod;
      // Fetch metadata binary
      var x = new XMLHttpRequest();
      x.open("GET", "/search-meta.bin", true);
      x.responseType = "arraybuffer";
      x.onload = function () {
        if (x.status === 200) {
          try {
            var data = new Uint8Array(x.response);
            wasmModule.oxidoc_search_init(data);
            indexLoaded = true;
          } catch (e) {
            console.error("[oxidoc-search] init failed:", e);
          }
        }
        cb();
      };
      x.onerror = function () { cb(); };
      x.send();
    }).catch(function (e) {
      console.error("[oxidoc-search] wasm load failed:", e);
      cb();
    });
  }

  // Fetch and load any chunks needed for the query
  function ensureChunks(query, cb) {
    if (!indexLoaded || !wasmModule) return cb();
    try {
      var neededJson = wasmModule.oxidoc_search_needed_chunks(query);
      var needed = JSON.parse(neededJson);
      var toLoad = needed.filter(function (id) { return !loadedChunks[id]; });
      if (toLoad.length === 0) return cb();

      var remaining = toLoad.length;
      toLoad.forEach(function (chunkId) {
        var x = new XMLHttpRequest();
        x.open("GET", "/search-chunk-" + chunkId + ".bin", true);
        x.responseType = "arraybuffer";
        x.onload = function () {
          if (x.status === 200) {
            try {
              var data = new Uint8Array(x.response);
              wasmModule.oxidoc_search_load_chunk(data);
              loadedChunks[chunkId] = true;
            } catch (e) {
              console.error("[oxidoc-search] chunk load failed:", e);
            }
          }
          remaining--;
          if (remaining === 0) cb();
        };
        x.onerror = function () {
          remaining--;
          if (remaining === 0) cb();
        };
        x.send();
      });
    } catch (e) {
      cb();
    }
  }

  function open() {
    overlay.hidden = false;
    document.body.style.overflow = "hidden";
    input.value = "";
    clearBtn.style.display = "none";
    resultsEl.innerHTML = "";
    noResultsEl.classList.remove("visible");
    emptyEl.hidden = false;
    activeIdx = -1;
    setTimeout(function () {
      input.focus();
    }, 50);
    ensureSearch(function () {});
  }

  function close() {
    overlay.hidden = true;
    document.body.style.overflow = "";
  }

  // Search via Wasm engine
  function search(query) {
    if (!indexLoaded || !wasmModule) return [];
    try {
      var json = wasmModule.oxidoc_search_query(query, 20);
      return JSON.parse(json);
    } catch (e) {
      return [];
    }
  }

  function highlight(text, query, highlightTerms) {
    if (!query) return text;
    // Use matched terms from the search engine (includes fuzzy matches)
    // plus the raw query terms as fallback
    var terms = query
      .toLowerCase()
      .split(/\s+/)
      .filter(function (t) {
        return t.length > 0;
      });
    if (highlightTerms && highlightTerms.length) {
      terms = terms.concat(highlightTerms);
    }
    // Deduplicate
    var seen = {};
    terms = terms.filter(function (t) {
      if (seen[t]) return false;
      seen[t] = true;
      return true;
    });
    var escaped = terms.map(function (t) {
      return t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    });
    if (!escaped.length) return text;
    var patterns = [];
    if (escaped.length > 1) {
      patterns.push(escaped.join("\\s*"));
    }
    patterns = patterns.concat(escaped);
    var re = new RegExp("(" + patterns.join("|") + ")", "gi");
    return text.replace(re, "<mark>$1</mark>");
  }

  function updateHighlights(results, query) {
    var items = resultsEl.querySelectorAll(".oxidoc-search-result");
    for (var i = 0; i < items.length && i < results.length; i++) {
      var doc = results[i];
      var ht = doc.highlight_terms || [];
      var bc = doc.breadcrumb || [];
      var titleEl = items[i].querySelector(".oxidoc-search-result-title");
      var snippetEl = items[i].querySelector(".oxidoc-search-result-snippet");
      if (titleEl) {
        var title = bc.length > 1 ? bc[bc.length - 1] : (bc.length === 1 ? bc[0] : doc.title);
        titleEl.innerHTML = highlight(title, query, ht);
      }
      if (snippetEl) {
        snippetEl.innerHTML = highlight(doc.snippet || "", query, ht);
      }
    }
  }

  function render(results, query) {
    // Skip re-render if results haven't changed (prevents icon blink)
    var key = results.map(function (r) { return r.path; }).join("|");
    if (key === lastResultKey && query) {
      // Same results, different query — update highlights in-place
      updateHighlights(results, query);
      return;
    }
    lastResultKey = key;

    resultsEl.innerHTML = "";
    activeIdx = -1;

    if (!query) {
      emptyEl.classList.remove("hidden");
      noResultsEl.classList.remove("visible");
      return;
    }
    emptyEl.classList.add("hidden");

    if (!results.length) {
      noResultsEl.classList.add("visible");
      return;
    }
    noResultsEl.classList.remove("visible");

    for (var i = 0; i < results.length; i++) {
      var doc = results[i];
      var a = document.createElement("a");
      a.href = doc.path;
      a.className = "oxidoc-search-result";
      a.addEventListener("click", function () { close(); });
      a.setAttribute("data-idx", i);
      var ht = doc.highlight_terms || [];
      var bc = doc.breadcrumb || [];
      var inner = "";
      if (bc.length > 1) {
        // Design A: breadcrumb trail (small) → closest heading (large) → snippet
        var trail = bc.slice(0, bc.length - 1).join(" › ");
        var heading = bc[bc.length - 1];
        inner =
          '<div class="oxidoc-search-result-page">' + trail + "</div>" +
          '<div class="oxidoc-search-result-title">' + highlight(heading, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      } else {
        // Design B: page title (large) → snippet
        var title = bc.length === 1 ? bc[0] : doc.title;
        inner =
          '<div class="oxidoc-search-result-title">' + highlight(title, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      }
      var icon = bc.length > 1
        ? "material-symbols:tag-rounded"
        : "material-symbols:description-rounded";
      a.innerHTML =
        '<iconify-icon icon="' + icon + '" width="18" height="18" class="oxidoc-search-result-icon"></iconify-icon>' +
        '<div class="oxidoc-search-result-content">' + inner + "</div>";
      resultsEl.appendChild(a);
    }
  }

  function setActive(idx) {
    var items = resultsEl.querySelectorAll(".oxidoc-search-result");
    if (activeIdx >= 0 && activeIdx < items.length) {
      items[activeIdx].classList.remove("active");
    }
    activeIdx = idx;
    if (activeIdx >= 0 && activeIdx < items.length) {
      items[activeIdx].classList.add("active");
      items[activeIdx].scrollIntoView({ block: "nearest" });
    }
  }

  // Event handlers
  input.addEventListener("input", function () {
    var q = input.value.trim();
    clearBtn.style.display = q ? "" : "none";
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(function () {
      ensureSearch(function () {
        ensureChunks(q, function () {
          var results = search(q);
          render(results, q);
        });
      });
    }, 150);
  });

  clearBtn.addEventListener("click", function () {
    input.value = "";
    clearBtn.style.display = "none";
    resultsEl.innerHTML = "";
    noResultsEl.classList.remove("visible");
    emptyEl.hidden = false;
    activeIdx = -1;
    input.focus();
  });

  closeBtn.addEventListener("click", close);
  overlay.addEventListener("click", function (e) {
    if (e.target === overlay) close();
  });

  // Keyboard navigation
  input.addEventListener("keydown", function (e) {
    var items = resultsEl.querySelectorAll(".oxidoc-search-result");
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (activeIdx < items.length - 1) setActive(activeIdx + 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (activeIdx > 0) setActive(activeIdx - 1);
    } else if (e.key === "Enter" && activeIdx >= 0 && activeIdx < items.length) {
      e.preventDefault();
      items[activeIdx].click();
    } else if (e.key === "Escape") {
      close();
    }
  });

  // Open triggers
  document.querySelectorAll("[data-oxidoc-search]").forEach(function (btn) {
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      open();
    });
  });

  // Ctrl+K / Cmd+K
  document.addEventListener("keydown", function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      if (overlay.hidden) {
        open();
      } else {
        close();
      }
    }
    if (e.key === "Escape" && !overlay.hidden) {
      close();
    }
  });
})();
