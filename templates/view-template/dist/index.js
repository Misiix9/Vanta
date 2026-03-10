(function(vanta) {
  function TemplateView($$anchor, $$props) {
    var api = $$props.api;
    var root = document.createElement('div');
    root.style.padding = '14px';
    root.style.display = 'grid';
    root.style.gap = '10px';
    root.innerHTML = '<h3 style="margin:0">SDK v2 Template</h3><p style="margin:0;opacity:.8">This is a starter extension view.</p>';
    var button = document.createElement('button');
    button.textContent = 'Show toast';
    button.onclick = function() { api.toast({ title: 'Template works', type: 'success' }); };
    root.appendChild(button);
    $$anchor.before(root);
  }

  vanta.registerExtension('my-view-extension', {
    commands: {
      'open': { component: TemplateView }
    }
  });
})(window.__vanta_host);
