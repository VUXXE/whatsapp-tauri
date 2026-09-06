// filebridge.js — injected into WhatsApp Web by the Tauri host.
//
// Provides:
//   window.__injectFiles(list)  — turn [{name, mime, b64}] into real File
//                                 objects and feed them to WhatsApp.
//   window.__report(msg)         — JS -> Rust debug channel (visible when
//                                 the app is launched from a terminal).

window.__report = (function () {
  var n = 0;
  return function (msg) {
    try {
      n++;
      window.location.href =
        'https://tauri-log.invalid/?n=' + n +
        '&t=' + Date.now() +
        '&m=' + encodeURIComponent(String(msg)).slice(0, 1500);
    } catch (e) {}
  };
})();

window.__injectFiles = async function (list) {
  __report('injectFiles called with ' + list.length + ' file(s)');
  var files = [];
  for (var i = 0; i < list.length; i++) {
    try {
      var res = await fetch('data:' + list[i].mime + ';base64,' + list[i].b64);
      var blob = await res.blob();
      files.push(new File([blob], list[i].name, { type: list[i].mime }));
    } catch (e) {
      __report('decode failed for ' + list[i].name + ': ' + e);
    }
  }
  if (!files.length) {
    __report('no files decoded, aborting');
    return;
  }

  var dt;
  try {
    dt = new DataTransfer();
    files.forEach(function (f) { dt.items.add(f); });
  } catch (e) {
    __report('DataTransfer unavailable: ' + e);
    return;
  }

  // Method 1: WhatsApp's own hidden file input (most reliable).
  var input = document.querySelector('input[type="file"]');
  if (!input) {
    var btn =
      document.querySelector('[data-testid="clip"]') ||
      document.querySelector('[data-icon="attach-menu-plus"]') ||
      document.querySelector('button[aria-label="Attach"]') ||
      document.querySelector('span[data-icon="plus"]');
    if (btn) {
      btn.click();
      __report('clicked attach button, waiting for file input');
      await new Promise(function (r) { setTimeout(r, 200); });
      input = document.querySelector('input[type="file"]');
    }
  }
  if (input) {
    try {
      input.files = dt.files;
      input.dispatchEvent(new Event('change', { bubbles: true }));
      input.dispatchEvent(new Event('input', { bubbles: true }));
      __report('fired change on file input for ' + files[0].name);
      return;
    } catch (e) {
      __report('file input injection failed: ' + e);
    }
  } else {
    __report('no file input found, trying synthetic events');
  }

  // Method 2: synthetic paste + drop events as fallback.
  var targets = [
    document.activeElement,
    document.querySelector('[data-testid="conversation-compose-box-input"]'),
    document.querySelector('div[contenteditable="true"]'),
    document.querySelector('#main'),
    document.body,
  ];
  targets.forEach(function (t) {
    if (!t) return;
    try {
      var pe = new Event('paste', { bubbles: true, cancelable: true });
      Object.defineProperty(pe, 'clipboardData', { get: function () { return dt; } });
      t.dispatchEvent(pe);
      var de = new DragEvent('drop', { bubbles: true, cancelable: true });
      Object.defineProperty(de, 'dataTransfer', { get: function () { return dt; } });
      t.dispatchEvent(de);
    } catch (e) {}
  });
  __report('dispatched synthetic paste/drop events');
};

// Paste bridge: WebKitGTK hides binary clipboard data from the page, so
// when a paste carries no image data, ask the Rust host to read the OS
// clipboard directly. Text pastes are left to the native path untouched.
document.addEventListener(
  'paste',
  function (e) {
    var items = e.clipboardData ? e.clipboardData.items : [];
    var hasImage = false;
    for (var i = 0; i < (items ? items.length : 0); i++) {
      if (items[i].type && items[i].type.indexOf('image/') === 0) {
        hasImage = true;
        break;
      }
    }
    if (!hasImage) {
      __report('paste without image data, triggering Rust bridge');
      window.location.href = 'https://tauri-clipboard.invalid/?t=' + Date.now();
    }
  },
  true
);

console.log('[filebridge] ready');
