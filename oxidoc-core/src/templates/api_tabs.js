// API page tab switching + copy
(function(){
  document.addEventListener('click', function(e) {
    // Copy button
    var copyBtn = e.target.closest('.oxidoc-api-copy-btn');
    if (copyBtn) {
      var container = copyBtn.closest('.oxidoc-api-snippets');
      if (!container) return;
      var activePanel = container.querySelector('.oxidoc-api-tab-panel.active');
      if (!activePanel) return;
      var code = activePanel.querySelector('code');
      var text = code ? code.textContent : activePanel.textContent;
      navigator.clipboard.writeText(text).then(function() {
        copyBtn.textContent = 'Copied!';
        setTimeout(function() { copyBtn.textContent = 'Copy'; }, 1500);
      });
      return;
    }

    // Tab switching (snippets, responses, response-examples)
    var tab = e.target.closest('.oxidoc-api-tab');
    if (!tab) return;
    var container = tab.closest('.oxidoc-api-snippets, .oxidoc-api-responses, .oxidoc-api-response-examples');
    if (!container) return;
    var id = tab.getAttribute('data-tab');
    container.querySelectorAll('.oxidoc-api-tab').forEach(function(t) {
      t.classList.remove('active');
      t.setAttribute('aria-selected', 'false');
    });
    container.querySelectorAll('.oxidoc-api-tab-panel').forEach(function(p) {
      p.classList.remove('active');
    });
    tab.classList.add('active');
    tab.setAttribute('aria-selected', 'true');
    var panel = container.querySelector('[data-panel="' + id + '"]');
    if (panel) panel.classList.add('active');
  });
})();
