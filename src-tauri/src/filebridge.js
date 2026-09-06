// filebridge.js — injected into WhatsApp Web by the Tauri host.
//
// Attach flow adapted from whatRust (MIT, github.com/karem505/whatRust):
// files are streamed from Rust as begin/chunk/end/commit messages keyed by
// a drop id, rebuilt into real File objects, and handed to WhatsApp's OWN
// attach inputs (NOT the always-mounted sticker input).
//
// Provides:
//   window.__dropFeed(msg) — Rust-side feed {op,drop,file,...}.
//   window.__report(msg)    — JS -> Rust debug channel (terminal logs).

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

(function () {
try {
  var drop = {};
  drop.log = function (m) {
    try { console.log('[drop] ' + m); } catch (e) {}
    try { window.__report('[drop] ' + String(m).slice(0, 280)); } catch (e) {}
  };
  drop.attachFailed = function () {
    drop.log('ATTACH FAILED — use the attach (+) button and try again');
  };

  // Fast base64 decode (Uint8Array.fromBase64 on modern WebKit, atob fallback).
  drop.b64ToBytes = function (b64) {
    if (typeof Uint8Array.fromBase64 === 'function') {
      return Uint8Array.fromBase64(b64);
    }
    var bin = atob(b64), n = bin.length, u = new Uint8Array(n);
    for (var i = 0; i < n; i++) u[i] = bin.charCodeAt(i);
    return u;
  };
  drop.b64DecodedSize = function (b64) {
    var n = String(b64 || '').length;
    if (!n) return 0;
    var pad = b64.slice(-2) === '==' ? 2 : (b64.slice(-1) === '=' ? 1 : 0);
    return Math.floor(n / 4) * 3 - pad;
  };

  // Incremental pump: decode queued stanzas a few at a time, yielding to
  // the main thread so WhatsApp stays responsive during big transfers.
  drop.pump = function (f) {
    return new Promise(function (resolve, reject) {
      (function step() {
        try {
          if (f.aborted) { f.b64s.length = 0; resolve(); return; }
          var b64s = f.b64s;
          var t0 = Date.now();
          while (b64s.length && Date.now() - t0 < 12) {
            var u = drop.b64ToBytes(b64s.shift());
            f.parts.push(u);
            f.got += u.length;
          }
          if (b64s.length) setTimeout(step, 0);
          else resolve();
        } catch (e) { reject(e); }
      })();
    });
  };
  drop.startPump = function (f) {
    if (f.error || f.aborted || f.pumpPromise || !f.b64s.length) {
      return f.pumpPromise || Promise.resolve();
    }
    f.pumpPromise = new Promise(function (resolve) { setTimeout(resolve, 0); })
      .then(function () { return drop.pump(f); })
      .catch(function (e) {
        f.error = e; f.b64s.length = 0; f.parts.length = 0; f.got = 0;
      })
      .then(function () {
        f.pumpPromise = null;
        if (!f.error && !f.aborted && f.b64s.length) return drop.startPump(f);
      });
    return f.pumpPromise;
  };
  drop.drain = function (f) {
    if (f.error) return Promise.reject(f.error);
    return drop.startPump(f).then(function () {
      if (f.error) throw f.error;
      if (f.pumpPromise || f.b64s.length) return drop.drain(f);
    });
  };
  drop.dataTransfer = function (files) {
    var dt = new DataTransfer();
    for (var i = 0; i < files.length; i++) dt.items.add(files[i]);
    return dt;
  };

  // Only these video types are accepted by WhatsApp's Photos & Videos input.
  drop.NATIVE_VIDEO = { 'video/mp4': 1, 'video/3gpp': 1, 'video/quicktime': 1 };
  drop.isMedia = function (type) {
    return /^image\//.test(type || '') || drop.NATIVE_VIDEO[type] === 1;
  };
  drop.qs = function (sels) {
    for (var i = 0; i < sels.length; i++) {
      var e = document.querySelector(sels[i]);
      if (e) return e;
    }
    return null;
  };
  // Media input = the file input whose accept lists a video type. The
  // always-mounted sticker input is image-only and can never match this.
  drop.findMediaInput = function () {
    var ins = document.querySelectorAll('input[type="file"]');
    for (var i = 0; i < ins.length; i++) {
      if (ins[i].isConnected !== false && /video/i.test(ins[i].accept || '')) return ins[i];
    }
    return null;
  };
  // Document input accepts everything (accept "" / "*" / no image+video).
  drop.findDocInput = function () {
    var ins = document.querySelectorAll('input[type="file"]');
    for (var i = 0; i < ins.length; i++) {
      if (ins[i].isConnected === false) continue;
      var a = (ins[i].accept || '').trim();
      if (a === '' || a === '*' || a === '*/*') return ins[i];
      if (!/image/i.test(a) && !/video/i.test(a)) return ins[i];
    }
    return null;
  };
  drop.openMenu = function () {
    var b = drop.qs([
      '[data-icon="plus"]',
      '[data-icon="attach-menu-plus"]',
      '[data-icon="clip"]',
      '[data-testid="clip"]',
      'button[title="Attach"]',
      '[aria-label="Attach"]',
      '[title="Attach"]',
    ]);
    if (!b) return false;
    (b.closest('button,[role="button"],div[role="button"]') || b).click();
    return true;
  };
  drop.clickMenuItem = function (kind) {
    var sels = kind === 'media'
      ? ['[data-testid="attach-image"]', '[data-icon="media-multiple"]', '[aria-label*="Photos"]', '[aria-label*="hoto"]']
      : ['[data-testid="attach-document"]', '[data-icon="document"]', '[aria-label*="Document"]', '[aria-label*="ocument"]'];
    var e = drop.qs(sels);
    if (!e) return false;
    (e.closest('li,button,[role="button"],div[role="button"]') || e).click();
    return true;
  };
  drop.poll = function (fn, ms) {
    return new Promise(function (resolve) {
      var t0 = Date.now();
      (function p() {
        var r = fn();
        if (r) return resolve(r);
        if (Date.now() - t0 >= ms) return resolve(null);
        setTimeout(p, 60);
      })();
    });
  };
  drop.inject = function (input, files) {
    var dt = drop.dataTransfer(files);
    try {
      input.files = dt.files; // settable in WebKit + Blink (WHATWG html#2861)
    } catch (e) {
      drop.log('input.files assign threw: ' + e);
      return false;
    }
    input.dispatchEvent(new Event('change', { bubbles: true }));
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  };
  // Open the attach menu (mounts lazily-rendered inputs) and inject into the
  // matching one. If the menu alone doesn't mount it, click the submenu item.
  drop.mountAndInject = function (kind, files) {
    var find = kind === 'media' ? drop.findMediaInput : drop.findDocInput;
    var existing = find();
    if (existing) {
      drop.log(kind + ' input already present');
      return Promise.resolve(drop.inject(existing, files));
    }
    drop.openMenu();
    return drop.poll(find, 1000).then(function (input) {
      if (input) return drop.inject(input, files);
      drop.clickMenuItem(kind);
      return drop.poll(find, 1000).then(function (input2) {
        if (input2) return drop.inject(input2, files);
        drop.log(kind + ' input NOT found after opening attach menu');
        return false;
      });
    });
  };
  // WhatsApp opens a media/document preview composer once attached — the
  // success signal, and the gate holding the NEXT queued drop.
  drop.composerOpen = function () {
    return !!(
      document.querySelector('[data-testid="media-caption-input-container"]') ||
      document.querySelector('[data-testid="media-editor"]') ||
      document.querySelector('[data-animate-modal-body="true"]') ||
      document.querySelector('span[data-icon="send"]') ||
      document.querySelector('span[data-icon="media-cancel"]')
    );
  };
  drop.waitFor = function (pred, ms) {
    return new Promise(function (resolve) {
      var t0 = Date.now();
      (function poll() {
        if (pred()) return resolve(true);
        if (Date.now() - t0 >= ms) return resolve(false);
        setTimeout(poll, 80);
      })();
    });
  };

  drop.pending = Object.create(null);
  drop.touch = function (st) {
    if (st.gc) clearTimeout(st.gc);
    st.gc = setTimeout(function () {
      delete drop.pending[st.id];
      drop.log('drop #' + st.id + ' expired uncommitted');
    }, 60000);
  };
  drop.state = function (id) {
    var st = drop.pending[id];
    if (!st) st = drop.pending[id] = { id: id, files: Object.create(null) };
    drop.touch(st);
    return st;
  };

  // Serialized injection queue: one composer at a time.
  drop.queue = Promise.resolve();
  drop.runQueued = function (job) {
    drop.queue = drop.queue.then(job, job);
  };

  drop.injectBatch = function (files) {
    var media = files.filter(function (f) { return drop.isMedia(f.type); });
    var docs = files.filter(function (f) { return !drop.isMedia(f.type); });
    // Mixed batch goes WHOLLY through the document input (accepts
    // everything) so no file is ever silently discarded.
    var kind, batch;
    if (media.length && docs.length) {
      kind = 'document'; batch = files;
      drop.log('mixed batch: routing all ' + files.length + ' file(s) as documents');
    } else if (media.length) {
      kind = 'media'; batch = media;
    } else {
      kind = 'document'; batch = docs;
    }
    drop.log('injecting ' + batch.length + ' file(s) via ' + kind + ' input');
    return drop.mountAndInject(kind, batch).then(function (ok) {
      drop.log(kind + ' inject ' + (ok ? 'dispatched' : 'FAILED'));
      return drop.waitFor(drop.composerOpen, 1800).then(function (open) {
        drop.log('composer ' + (open ? 'opened' : 'NOT detected') + ' (' + kind + ')');
        if (!open) return;
        return drop.waitFor(function () { return !drop.composerOpen(); }, 120000);
      });
    });
  };

  // Rust-side feed: begin -> chunk* -> end|abort, then commit.
  window.__dropFeed = function (msg) {
    try {
      if (!msg || typeof msg.drop !== 'number') return 'BADMSG';
      var st = drop.state(msg.drop);
      if (msg.op === 'begin') {
        st.files[msg.file] = {
          name: String(msg.name || 'file'),
          type: String(msg.type || 'application/octet-stream'),
          size: msg.size >>> 0,
          b64s: [], parts: [], got: 0,
          done: false, aborted: false, error: null, pumpPromise: null,
        };
        return 'OK';
      }
      var f = st.files[msg.file];
      if (msg.op === 'chunk') {
        if (!f || f.done) return 'NOFILE';
        var ps = msg.parts || (msg.b64 ? [msg.b64] : []);
        for (var i = 0; i < ps.length; i++) f.b64s.push(ps[i]);
        drop.startPump(f);
        return 'OK';
      }
      if (msg.op === 'end') {
        if (f) f.done = true;
        return 'OK';
      }
      if (msg.op === 'abort') {
        if (f) { f.aborted = true; f.b64s.length = 0; f.parts.length = 0; }
        delete st.files[msg.file];
        drop.log('drop #' + st.id + ' file ' + msg.file + ' aborted by sender');
        return 'OK';
      }
      if (msg.op === 'commit') {
        if (st.gc) clearTimeout(st.gc);
        delete drop.pending[st.id];
        var entries = Object.keys(st.files).map(function (k) { return st.files[k]; });
        var hadDecodeError = false;
        var valid = entries.filter(function (e) {
          if (e.error) {
            hadDecodeError = true;
            drop.log('drop #' + st.id + ' decode failed before commit: ' + e.error);
            return false;
          }
          var received = e.got + e.b64s.reduce(function (n, b64) {
            return n + drop.b64DecodedSize(b64);
          }, 0);
          if (!e.done || received !== e.size) {
            drop.log('drop #' + st.id + ' file incomplete (' + received + '/' + e.size + '), skipped');
            return false;
          }
          return true;
        });
        if (valid.length === 0) return hadDecodeError ? 'ERR' : 'EMPTY';
        if (hadDecodeError) drop.attachFailed();
        drop.runQueued(function () {
          var files = [];
          return valid.reduce(function (p, e) {
            return p.then(function () {
              return drop.drain(e).then(function () {
                if (e.got !== e.size) {
                  drop.log('drop #' + st.id + ' decode size mismatch (' + e.got + '/' + e.size + '), skipped');
                  return;
                }
                files.push(new File(e.parts, e.name, { type: e.type }));
                e.parts = null; e.b64s = null;
              });
            });
          }, Promise.resolve()).then(function () {
            if (files.length === 0) {
              drop.log('drop #' + st.id + ' had no complete files');
              return;
            }
            return drop.injectBatch(files).catch(function (e) {
              drop.log('inject error: ' + e);
              drop.attachFailed();
            });
          }).catch(function (e) {
            drop.log('decode error: ' + e);
            drop.attachFailed();
          });
        });
        return 'QUEUED:' + valid.length;
      }
      return 'BADOP';
    } catch (e) {
      drop.log('feed error: ' + e);
      return 'ERR';
    }
  };
  drop.log('drop injector ready (chunked feed, routed media/doc inputs)');
} catch (e) {}
})();

// Paste bridge: WebKitGTK hides binary clipboard data from the page, so when
// a paste carries no image data, ask the Rust host to read the OS clipboard
// directly. Text pastes are left to the native path untouched.
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
      window.__report('paste without image data, triggering Rust bridge');
      window.location.href = 'https://tauri-clipboard.invalid/?t=' + Date.now();
    }
  },
  true
);

console.log('[filebridge] ready');
