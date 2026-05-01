(function () {
  function copyText(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      return navigator.clipboard.writeText(text);
    }
    return new Promise(function (resolve, reject) {
      var ta = document.createElement('textarea');
      ta.value = text;
      ta.setAttribute('readonly', '');
      ta.style.position = 'absolute';
      ta.style.left = '-9999px';
      document.body.appendChild(ta);
      ta.select();
      try {
        document.execCommand('copy');
        resolve();
      } catch (e) {
        reject(e);
      } finally {
        document.body.removeChild(ta);
      }
    });
  }

  function flashLabel(button, text, success) {
    var label = button.querySelector('.oxidoc-llm-copy-label');
    if (!label) return;
    var original = label.textContent;
    if (success) button.classList.add('oxidoc-llm-copied');
    label.textContent = text;
    setTimeout(function () {
      button.classList.remove('oxidoc-llm-copied');
      label.textContent = original;
    }, 1800);
  }

  function handleCopy(button) {
    var wrapper = button.closest('.oxidoc-llm-actions');
    if (!wrapper) return;
    var source = wrapper.querySelector('.oxidoc-llm-source');
    if (!source) {
      flashLabel(button, 'Copy failed', false);
      return;
    }
    copyText(source.textContent || '')
      .then(function () { flashLabel(button, '✓ Copied', true); })
      .catch(function () { flashLabel(button, 'Copy failed', false); });
  }

  function closeAllMenus(except) {
    var open = document.querySelectorAll('.oxidoc-llm-actions.oxidoc-llm-open');
    for (var i = 0; i < open.length; i++) {
      if (open[i] !== except) {
        open[i].classList.remove('oxidoc-llm-open');
        var t = open[i].querySelector('.oxidoc-llm-toggle');
        var m = open[i].querySelector('.oxidoc-llm-menu');
        if (t) t.setAttribute('aria-expanded', 'false');
        if (m) m.setAttribute('hidden', '');
      }
    }
  }

  function toggleMenu(wrapper) {
    var toggle = wrapper.querySelector('.oxidoc-llm-toggle');
    var menu = wrapper.querySelector('.oxidoc-llm-menu');
    if (!toggle || !menu) return;
    var open = wrapper.classList.toggle('oxidoc-llm-open');
    toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    if (open) {
      menu.removeAttribute('hidden');
      closeAllMenus(wrapper);
    } else {
      menu.setAttribute('hidden', '');
    }
  }

  document.addEventListener('click', function (e) {
    var target = e.target;
    if (!(target instanceof Element)) return;

    var copyBtn = target.closest('.oxidoc-llm-copy');
    if (copyBtn) {
      e.preventDefault();
      handleCopy(copyBtn);
      return;
    }

    var toggleBtn = target.closest('.oxidoc-llm-toggle');
    if (toggleBtn) {
      e.preventDefault();
      var wrapper = toggleBtn.closest('.oxidoc-llm-actions');
      if (wrapper) toggleMenu(wrapper);
      return;
    }

    if (!target.closest('.oxidoc-llm-actions')) {
      closeAllMenus(null);
    }
  });

  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape') closeAllMenus(null);
  });
})();
