// powertools.js for WhatsApp Tauri

(function() {
    console.log("%c WhatsApp Tauri Powertools Injected ", "background: #25D366; color: white; font-weight: bold; font-size: 14px; padding: 4px; border-radius: 4px;");

    // 1. Dark Mode CSS Enforce & Privacy Blur CSS
    const styleBlock = document.createElement('style');
    styleBlock.innerHTML = `
        /* Dark Mode Enforcement */
        body:not(.dark) {
            background-color: #111b21 !important;
            color: #e9edef !important;
        }

        /* Privacy Blur Mode */
        body.privacy-blur [data-testid="msg-container"],
        body.privacy-blur [data-testid="conversation-panel-wrapper"] header,
        body.privacy-blur [data-testid="chat-list"],
        body.privacy-blur header {
            filter: blur(6px) grayscale(100%);
            transition: filter 0.3s ease-in-out, transform 0.3s ease-in-out;
            opacity: 0.8;
        }

        body.privacy-blur [data-testid="msg-container"]:hover,
        body.privacy-blur [data-testid="conversation-panel-wrapper"] header:hover,
        body.privacy-blur [data-testid="chat-list"]:hover,
        body.privacy-blur header:hover {
            filter: blur(0px) grayscale(0%);
            opacity: 1;
        }

        body.privacy-blur img {
            filter: blur(10px);
        }
        
        body.privacy-blur img:hover {
            filter: blur(0px);
        }
    `;
    
    // Ensure document.head is ready
    const initStyles = setInterval(() => {
        if (document.head) {
            document.head.appendChild(styleBlock);
            console.log("%c[Powertools]%c CSS overrides injected (Dark Mode prep, Privacy Blur)", "color: #25D366; font-weight: bold;", "color: inherit;");
            clearInterval(initStyles);
        }
    }, 100);

    // Enforce dark mode class on body/html
    const enableDarkMode = () => {
        if (document.body && !document.body.classList.contains('dark')) {
            document.body.classList.add('dark');
        }
        if (document.documentElement && !document.documentElement.classList.contains('dark')) {
            document.documentElement.classList.add('dark');
        }
    };
    
    // Observer to re-apply dark class if WhatsApp removes it or replaces body
    const observer = new MutationObserver(() => {
        if (document.body) enableDarkMode();
    });
    
    const initDarkObserver = setInterval(() => {
        if (document.body) {
            enableDarkMode();
            observer.observe(document.body, { attributes: true, attributeFilter: ['class'] });
            if (document.documentElement) {
                observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
            }
            console.log("%c[Powertools]%c Dark mode enforced.", "color: #25D366; font-weight: bold;", "color: inherit;");
            clearInterval(initDarkObserver);
        }
    }, 500);


    // 2 & 3. Keybindings: Privacy Blur Mode & UI Zoom Scaling
    let currentZoom = 1.0;
    const ZOOM_STEP = 0.1;

    const setZoom = (zoomLevel) => {
        // Clamp between 0.8 and 1.5 per requirements
        currentZoom = Math.min(Math.max(zoomLevel, 0.8), 1.5);
        if (document.body) {
            document.body.style.zoom = currentZoom;
            console.log(`%c[Powertools]%c Zoom set to ${Math.round(currentZoom * 100)}%`, "color: #25D366; font-weight: bold;", "color: inherit;");
        }
    };

    window.addEventListener('keydown', (e) => {
        // Privacy Blur: Ctrl + Alt + P
        if (e.ctrlKey && e.altKey && e.code === 'KeyP') {
            e.preventDefault();
            if (document.body) {
                document.body.classList.toggle('privacy-blur');
                const isEnabled = document.body.classList.contains('privacy-blur');
                console.log(`%c[Powertools]%c Privacy Blur Mode ${isEnabled ? 'ENABLED' : 'DISABLED'}`, "color: #25D366; font-weight: bold;", "color: inherit;");
            }
        }

        // Zoom In: Ctrl + Plus (Numpad) or Ctrl + Equal (Standard keyboard)
        if (e.ctrlKey && (e.code === 'NumpadAdd' || e.code === 'Equal')) {
            e.preventDefault();
            setZoom(currentZoom + ZOOM_STEP);
        }

        // Zoom Out: Ctrl + Minus
        if (e.ctrlKey && (e.code === 'NumpadSubtract' || e.code === 'Minus')) {
            e.preventDefault();
            setZoom(currentZoom - ZOOM_STEP);
        }

        // Zoom Reset: Ctrl + Zero
        if (e.ctrlKey && (e.code === 'Numpad0' || e.code === 'Digit0')) {
            e.preventDefault();
            setZoom(1.0);
        }
    // Need to use capture phase to ensure WhatsApp doesn't block the shortcuts
    }, true); 
    console.log("%c[Powertools]%c Keybindings initialized (Privacy: Ctrl+Alt+P, Zoom: Ctrl++/-/0)", "color: #25D366; font-weight: bold;", "color: inherit;");

    // 4. Auto-grant Notification API permissions
    if ("Notification" in window) {
        // Mock the permission property
        Object.defineProperty(Notification, 'permission', {
            get: () => 'granted',
            configurable: true,
            enumerable: true
        });
        
        // Mock the requestPermission method
        Notification.requestPermission = function(callback) {
            const promise = Promise.resolve('granted');
            if (callback && typeof callback === 'function') {
                promise.then(callback);
            }
            return promise;
        };
        console.log("%c[Powertools]%c Notification permissions auto-granted.", "color: #25D366; font-weight: bold;", "color: inherit;");
    }
})();
