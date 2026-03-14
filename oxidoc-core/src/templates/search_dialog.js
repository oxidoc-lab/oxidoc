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
  var semanticReady = false;
  var semanticLoading = false;
  var semanticEnabled = false; // set true when search-vectors.json exists (semantic=true in config)
  var activeIdx = -1;
  var debounceTimer = null;
  var lastResultKey = "";
  var loadedChunks = {};
  var aiMode = false; // true when user clicked "Ask AI"

  var TAG_ICON = '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" class="oxidoc-search-result-icon"><path fill="currentColor" d="m9 16l-.825 3.275q-.075.325-.325.525t-.6.2q-.475 0-.775-.375T6.3 18.8L7 16H4.275q-.5 0-.8-.387T3.3 14.75q.075-.35.35-.55t.625-.2H7.5l1-4H5.775q-.5 0-.8-.387T4.8 8.75q.075-.35.35-.55t.625-.2H9l.825-3.275Q9.9 4.4 10.15 4.2t.6-.2q.475 0 .775.375t.175.825L11 8h4l.825-3.275q.075-.325.325-.525t.6-.2q.475 0 .775.375t.175.825L17 8h2.725q.5 0 .8.387t.175.863q-.075.35-.35.55t-.625.2H16.5l-1 4h2.725q.5 0 .8.388t.175.862q-.075.35-.35.55t-.625.2H15l-.825 3.275q-.075.325-.325.525t-.6.2q-.475 0-.775-.375T12.3 18.8L13 16zm.5-2h4l1-4h-4z"/></svg>';
  var DESC_ICON = '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" class="oxidoc-search-result-icon"><path fill="currentColor" d="M9 18h6q.425 0 .713-.288T16 17t-.288-.712T15 16H9q-.425 0-.712.288T8 17t.288.713T9 18m0-4h6q.425 0 .713-.288T16 13t-.288-.712T15 12H9q-.425 0-.712.288T8 13t.288.713T9 14m-3 8q-.825 0-1.412-.587T4 20V4q0-.825.588-1.412T6 2h7.175q.4 0 .763.15t.637.425l4.85 4.85q.275.275.425.638t.15.762V20q0 .825-.587 1.413T18 22zm7-14q0 .425.288.713T14 9h4l-5-5z"/></svg>';

  // Probe whether semantic assets exist (non-blocking HEAD request)
  function detectSemantic() {
    var x = new XMLHttpRequest();
    x.open("HEAD", "/search-vectors.json", true);
    x.onload = function () {
      if (x.status === 200) {
        semanticEnabled = true;
        // Re-render current results to show "Ask AI" row if query is active
        var q = input.value.trim();
        if (q && !aiMode) {
          lastResultKey = ""; // force re-render
          ensureSearch(function () {
            ensureChunks(q, function () {
              render(searchLexical(q), q);
            });
          });
        }
      }
    };
    x.send();
  }

  // Load the Wasm search module and initialize with the metadata
  function ensureSearch(cb) {
    if (indexLoaded) return cb();
    var loadWasm = window.__oxidoc_search ? window.__oxidoc_search() : Promise.reject("no loader");
    loadWasm.then(function (mod) {
      wasmModule = mod;
      var x = new XMLHttpRequest();
      x.open("GET", "/search-meta.bin", true);
      x.responseType = "arraybuffer";
      x.onload = function () {
        if (x.status === 200) {
          try {
            var data = new Uint8Array(x.response);
            wasmModule.oxidoc_search_init(data);
            indexLoaded = true;
            detectSemantic();
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

  // Load semantic resources (model + vectors). Called on demand when user clicks "Ask AI".
  function loadSemantic(cb) {
    if (semanticReady) return cb();
    if (semanticLoading) {
      // Already loading — queue callback
      var check = setInterval(function () {
        if (semanticReady) { clearInterval(check); cb(); }
      }, 100);
      return;
    }
    semanticLoading = true;

    var vectorsReq = new XMLHttpRequest();
    vectorsReq.open("GET", "/search-vectors.json", true);

    var modelReq = new XMLHttpRequest();
    modelReq.open("GET", "/search-model.gguf", true);
    modelReq.responseType = "arraybuffer";

    var vectorsText = null;
    var modelData = null;
    var done = 0;

    function tryInit() {
      done++;
      if (done < 2) return;
      if (!vectorsText || !modelData) { cb(); return; }
      try {
        wasmModule.oxidoc_search_load_semantic(new Uint8Array(modelData), vectorsText);
        semanticReady = true;
        console.log("[oxidoc-search] Semantic search enabled");
      } catch (e) {
        console.warn("[oxidoc-search] Semantic init failed:", e);
      }
      cb();
    }

    vectorsReq.onload = function () {
      if (vectorsReq.status === 200) vectorsText = vectorsReq.responseText;
      tryInit();
    };
    vectorsReq.onerror = function () { done++; tryInit(); };
    modelReq.onload = function () {
      if (modelReq.status === 200) modelData = modelReq.response;
      tryInit();
    };
    modelReq.onerror = function () { done++; tryInit(); };

    vectorsReq.send();
    modelReq.send();
  }

  function open() {
    overlay.hidden = false;
    document.body.style.overflow = "hidden";
    input.value = "";
    clearBtn.style.display = "none";
    resultsEl.innerHTML = "";
    noResultsEl.classList.remove("visible");
    emptyEl.classList.remove("hidden");
    activeIdx = -1;
    aiMode = false;
    setTimeout(function () {
      input.focus();
    }, 50);
    ensureSearch(function () {});
  }

  function close() {
    overlay.hidden = true;
    document.body.style.overflow = "";
  }

  // Lexical-only search
  function searchLexical(query) {
    if (!indexLoaded || !wasmModule) return [];
    try {
      var json = wasmModule.oxidoc_search_query(query, 20);
      return JSON.parse(json);
    } catch (e) {
      return [];
    }
  }

  // Hybrid AI search (lexical + semantic via RRF)
  function searchAI(query) {
    if (!indexLoaded || !wasmModule) return [];
    try {
      var json = wasmModule.oxidoc_search_query_ai(query, 20);
      return JSON.parse(json);
    } catch (e) {
      return [];
    }
  }

  function escapeHtml(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function highlight(text, query, highlightTerms) {
    if (!query) return text;
    var terms = query
      .toLowerCase()
      .split(/\s+/)
      .filter(function (t) {
        return t.length > 0;
      });
    if (highlightTerms && highlightTerms.length) {
      terms = terms.concat(highlightTerms);
    }
    var seen = {};
    terms = terms.filter(function (t) {
      if (seen[t]) return false;
      seen[t] = true;
      return true;
    });
    var escaped = terms.map(function (t) {
      return t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    });
    if (!escaped.length) return escapeHtml(text);
    text = escapeHtml(text);
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

  // Create the "Ask AI" row element
  function createAskAIRow(query) {
    var row = document.createElement("button");
    row.className = "oxidoc-search-ask-ai";
    row.type = "button";
    row.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" class="oxidoc-search-ask-ai-icon"><path fill="currentColor" d="m19 9l-1.25-2.75L15 5l2.75-1.25L19 1l1.25 2.75L23 5l-2.75 1.25L19 9Zm0 14l-1.25-2.75L15 19l2.75-1.25L19 15l1.25 2.75L23 19l-2.75 1.25L19 23ZM9 20l-2.5-5.5L1 12l5.5-2.5L9 4l2.5 5.5L17 12l-5.5 2.5L9 20Z"/></svg>' +
      '<span class="oxidoc-search-ask-ai-label">Search using AI:</span> ' +
      '<span class="oxidoc-search-ask-ai-query">' + escapeHtml(query) + '</span>';
    row.addEventListener("click", function () {
      activateAI(query);
    });
    return row;
  }

  // Activate AI search mode
  function activateAI(query) {
    aiMode = true;
    lastResultKey = "";
    resultsEl.innerHTML = "";
    noResultsEl.classList.remove("visible");
    emptyEl.classList.add("hidden");

    // Show loading state
    var loading = document.createElement("div");
    loading.className = "oxidoc-search-ai-loading";
    loading.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" class="oxidoc-search-ask-ai-icon"><path fill="currentColor" d="m19 9l-1.25-2.75L15 5l2.75-1.25L19 1l1.25 2.75L23 5l-2.75 1.25L19 9Zm0 14l-1.25-2.75L15 19l2.75-1.25L19 15l1.25 2.75L23 19l-2.75 1.25L19 23ZM9 20l-2.5-5.5L1 12l5.5-2.5L9 4l2.5 5.5L17 12l-5.5 2.5L9 20Z"/></svg>' +
      '<span>Loading AI search...</span>';
    resultsEl.appendChild(loading);

    ensureChunks(query, function () {
      loadSemantic(function () {
        if (!semanticReady) {
          loading.querySelector("span").textContent = "AI search unavailable";
          return;
        }
        var results = searchAI(query);
        renderAIResults(results, query);
      });
    });
  }

  // Render AI search results (no "Ask AI" row, show "back to results" instead)
  function renderAIResults(results, query) {
    resultsEl.innerHTML = "";
    activeIdx = -1;

    // AI results banner
    var banner = document.createElement("div");
    banner.className = "oxidoc-search-ai-banner";
    banner.innerHTML =
      '<button type="button" class="oxidoc-search-ai-banner-back">' +
        '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"><path fill="currentColor" d="m7.825 13l4.9 4.9q.3.3.288.7t-.313.7q-.3.275-.7.288t-.7-.288l-6.6-6.6q-.15-.15-.213-.325T4.426 12t.063-.375t.212-.325l6.6-6.6q.275-.275.688-.275t.712.275q.3.3.3.713t-.3.712L7.825 11H19q.425 0 .713.288T20 12t-.288.713T19 13z"/></svg>' +
      '</button>' +
      '<div class="oxidoc-search-ai-banner-label">' +
        '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" d="m19 9l-1.25-2.75L15 5l2.75-1.25L19 1l1.25 2.75L23 5l-2.75 1.25L19 9Zm0 14l-1.25-2.75L15 19l2.75-1.25L19 15l1.25 2.75L23 19l-2.75 1.25L19 23ZM9 20l-2.5-5.5L1 12l5.5-2.5L9 4l2.5 5.5L17 12l-5.5 2.5L9 20Z"/></svg>' +
        '<span>AI search results</span>' +
      '</div>';
    banner.querySelector(".oxidoc-search-ai-banner-back").addEventListener("click", function () {
      aiMode = false;
      lastResultKey = "";
      var q = input.value.trim();
      ensureChunks(q, function () {
        render(searchLexical(q), q);
      });
      input.focus();
    });
    resultsEl.appendChild(banner);

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
        var trail = bc.slice(0, bc.length - 1).map(escapeHtml).join(" \u203A ");
        var heading = bc[bc.length - 1];
        inner =
          '<div class="oxidoc-search-result-page">' + trail + "</div>" +
          '<div class="oxidoc-search-result-title">' + highlight(heading, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      } else {
        var title = bc.length === 1 ? bc[0] : doc.title;
        inner =
          '<div class="oxidoc-search-result-title">' + highlight(title, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      }
      var icon = bc.length > 1 ? TAG_ICON : DESC_ICON;
      a.innerHTML =
        icon +
        '<div class="oxidoc-search-result-content">' + inner + "</div>";
      resultsEl.appendChild(a);
    }
  }

  function render(results, query) {
    if (aiMode) return; // don't overwrite AI results

    var key = results.map(function (r) { return r.path; }).join("|");
    if (key === lastResultKey && query) {
      // Update AI row query text
      var aiRow = resultsEl.querySelector(".oxidoc-search-ask-ai");
      if (aiRow) {
        aiRow.querySelector(".oxidoc-search-ask-ai-query").textContent = query;
        aiRow.onclick = function () { activateAI(query); };
      }
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

    // "Ask AI" row at top (only if semantic is enabled)
    if (semanticEnabled) {
      resultsEl.appendChild(createAskAIRow(query));
    }

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
        var trail = bc.slice(0, bc.length - 1).map(escapeHtml).join(" \u203A ");
        var heading = bc[bc.length - 1];
        inner =
          '<div class="oxidoc-search-result-page">' + trail + "</div>" +
          '<div class="oxidoc-search-result-title">' + highlight(heading, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      } else {
        var title = bc.length === 1 ? bc[0] : doc.title;
        inner =
          '<div class="oxidoc-search-result-title">' + highlight(title, query, ht) + "</div>" +
          '<div class="oxidoc-search-result-snippet">' + highlight(doc.snippet || "", query, ht) + "</div>";
      }
      var icon = bc.length > 1 ? TAG_ICON : DESC_ICON;
      a.innerHTML =
        icon +
        '<div class="oxidoc-search-result-content">' + inner + "</div>";
      resultsEl.appendChild(a);
    }
  }

  function setActive(idx) {
    var items = resultsEl.querySelectorAll(".oxidoc-search-result, .oxidoc-search-ask-ai");
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
    aiMode = false;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(function () {
      ensureSearch(function () {
        ensureChunks(q, function () {
          var results = searchLexical(q);
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
    emptyEl.classList.remove("hidden");
    activeIdx = -1;
    aiMode = false;
    input.focus();
  });

  closeBtn.addEventListener("click", close);
  overlay.addEventListener("click", function (e) {
    if (e.target === overlay) close();
  });

  // Keyboard navigation
  input.addEventListener("keydown", function (e) {
    var items = resultsEl.querySelectorAll(".oxidoc-search-result, .oxidoc-search-ask-ai");
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
