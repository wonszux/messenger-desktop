import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';

const script = readFileSync(new URL('../src-tauri/src/links.js', import.meta.url), 'utf8');
const blobUrl = 'blob:https://www.messenger.com/attachment-id';

// Exercise the injected production script. These records are download requests,
// not successful downloads. Browser policy, IPC and native UI need WebView2 checks.
function webview() {
  const listeners = new Map();
  const pageListeners = new Map();
  const downloadRequests = [];
  const children = new Set();
  const window = {
    location: { href: 'https://www.messenger.com/t/123' },
    addEventListener(type, listener) {
      const handlers = pageListeners.get(type) ?? [];
      handlers.push(listener);
      pageListeners.set(type, handlers);
    },
  };

  function anchor(href, { target = '', download } = {}) {
    const link = {
      tagName: 'A', href, target,
      get protocol() { return new URL(this.href).protocol; },
      hasAttribute(name) { return name === 'download' && Object.hasOwn(this, 'download'); },
      click() { click(this); },
      remove() { children.delete(this); },
    };
    if (download !== undefined) link.download = download;
    return link;
  }

  function click(link, options = {}) {
    const event = {
      type: 'click', button: 0, defaultPrevented: false, stopped: false,
      composedPath: () => [{ tagName: 'SPAN' }, link],
      preventDefault() { this.defaultPrevented = true; },
      stopImmediatePropagation() { this.stopped = true; },
      ...options,
    };
    listeners.get(event.type)(event);
    for (const handler of pageListeners.get(event.type) ?? []) {
      if (event.stopped) break;
      handler(event);
    }
    if (!event.defaultPrevented && link.hasAttribute('download')) {
      downloadRequests.push({ href: link.href, name: link.download });
    }
    return event;
  }

  const document = {
    baseURI: window.location.href,
    addEventListener(type, listener) { listeners.set(type, listener); },
    createElement(tag) { assert.equal(tag, 'a'); return anchor(''); },
    documentElement: { appendChild(link) { children.add(link); } },
  };
  vm.runInNewContext(script, { document, window, URL });
  return { window, anchor, click, downloadRequests, children };
}

for (const name of ['faktura ąę.pdf', '']) {
  test(`existing blob download preserves its filename (${JSON.stringify(name)})`, () => {
    const app = webview();
    const event = app.click(app.anchor(blobUrl, { target: '_blank', download: name }));
    assert.equal(event.defaultPrevented, false);
    assert.equal(app.window.location.href, 'https://www.messenger.com/t/123');
    assert.deepEqual(app.downloadRequests, [{ href: blobUrl, name }]);
  });
}

for (const target of ['', '_blank']) {
  test(`blob link requests one download without replacing the conversation (target=${target})`, () => {
    const app = webview();
    const event = app.click(app.anchor(blobUrl, { target }));
    assert.equal(event.defaultPrevented, true);
    assert.equal(event.stopped, true);
    assert.deepEqual(app.downloadRequests, [{ href: blobUrl, name: '' }]);
    assert.equal(app.children.size, 0);
    assert.equal(app.window.location.href, 'https://www.messenger.com/t/123');
  });
}

test('window.open(blob) triggers a download, never blob navigation', () => {
  const app = webview();
  assert.equal(app.window.open(blobUrl), null);
  assert.deepEqual(app.downloadRequests, [{ href: blobUrl, name: '' }]);
  assert.equal(app.window.location.href, 'https://www.messenger.com/t/123');
});

test('new-tab Facebook click reaches the native navigation policy once', () => {
  const app = webview();
  const url = 'https://www.facebook.com/?locale=pl_PL';
  const event = app.click(app.anchor(url, { target: '_blank' }));
  assert.equal(event.defaultPrevented, true);
  assert.equal(event.stopped, true);
  assert.equal(app.window.location.href, url);
  assert.deepEqual(app.downloadRequests, []);
});

for (const modifiers of [{ type: 'auxclick', button: 1 }, { ctrlKey: true }, { metaKey: true }, { shiftKey: true }]) {
  test(`modified click reaches native navigation (${JSON.stringify(modifiers)})`, () => {
    const app = webview();
    const url = 'https://example.com/';
    assert.equal(app.click(app.anchor(url), modifiers).defaultPrevented, true);
    assert.equal(app.window.location.href, url);
  });
}

test('right click and already handled events do not launch a second action', () => {
  const app = webview();
  const link = app.anchor(blobUrl);
  assert.equal(app.click(link, { type: 'auxclick', button: 2 }).defaultPrevented, false);
  app.click(link, { defaultPrevented: true });
  assert.deepEqual(app.downloadRequests, []);
  assert.equal(app.window.location.href, 'https://www.messenger.com/t/123');
});

test('ordinary conversation links retain normal page behavior', () => {
  const app = webview();
  const event = app.click(app.anchor('https://www.messenger.com/t/456'));
  assert.equal(event.defaultPrevented, false);
  assert.equal(event.stopped, false);
});

test('HTTP download requests preserve metadata and reach page handlers', () => {
  const app = webview();
  const url = 'https://www.messenger.com/attachment';
  let pageClicks = 0;
  app.window.addEventListener('click', event => {
    pageClicks++;
    assert.equal(event.defaultPrevented, false);
  });
  const event = app.click(app.anchor(url, { target: '_blank', download: 'document.pdf' }));
  assert.equal(event.defaultPrevented, false);
  assert.equal(event.stopped, false);
  assert.equal(pageClicks, 1);
  assert.deepEqual(app.downloadRequests, [{ href: url, name: 'document.pdf' }]);
  assert.equal(app.window.location.href, 'https://www.messenger.com/t/123');
});

test('page handlers can cancel download requests', () => {
  const app = webview();
  app.window.addEventListener('click', event => event.preventDefault());
  const event = app.click(app.anchor(blobUrl, { download: 'document.pdf' }));
  assert.equal(event.defaultPrevented, true);
  assert.deepEqual(app.downloadRequests, []);
});

test('intercepted links do not reach a second page click handler', () => {
  const app = webview();
  let pageClicks = 0;
  app.window.addEventListener('click', () => pageClicks++);
  app.click(app.anchor('https://example.com/', { target: '_blank' }));
  assert.equal(pageClicks, 0);
  assert.equal(app.window.location.href, 'https://example.com/');
});

test('window.open resolves relative URLs before native routing', () => {
  const app = webview();
  app.window.open('/t/456');
  assert.equal(app.window.location.href, 'https://www.messenger.com/t/456');
});
