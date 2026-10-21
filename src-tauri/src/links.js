(() => {
  function downloadBlob(url) {
    const link = document.createElement('a');
    link.href = url;
    // Let WebView choose the name/extension from the blob's MIME type.
    // Never invent a .pdf extension for arbitrary data.
    link.download = '';
    document.documentElement.appendChild(link);
    link.click();
    link.remove();
  }

  function interceptLink(event) {
    if (event.defaultPrevented || (event.type === 'auxclick' && event.button !== 1)) return;
    const link = event.composedPath().find(node => node.tagName === 'A' && node.href);
    if (!link) return;

    // Keep real download links intact, including download="" and their filename.
    // Turning these into navigation loses both the download intent and the name.
    if (link.hasAttribute('download')) return;

    if (link.protocol === 'blob:') {
      event.preventDefault();
      event.stopImmediatePropagation();
      downloadBlob(link.href);
      return;
    }

    if (link.target === '_blank' || event.type === 'auxclick' || event.ctrlKey || event.metaKey || event.shiftKey) {
      event.preventDefault();
      event.stopImmediatePropagation();
      // Rust applies the same policy to normal, modified and new-tab clicks.
      window.location.href = link.href;
    }
  }

  document.addEventListener('click', interceptLink, true);
  document.addEventListener('auxclick', interceptLink, true);

  window.open = function (url) {
    if (url) {
      const destination = new URL(String(url), document.baseURI);
      if (destination.protocol === 'blob:') downloadBlob(destination.href);
      else window.location.href = destination.href;
    }
    return null;
  };
})();
