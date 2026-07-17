(function () {
  function loadApp() {
    var src = window.__MARKFLY_ENTRY__
    if (!src) return
    var script = document.createElement('script')
    script.type = 'module'
    script.crossOrigin = 'anonymous'
    script.src = src
    document.body.appendChild(script)
  }

  function needsFullPreviewLite(content) {
    if (!content) return false
    if (/^\s*---\r?\n[\s\S]*?\r?\n---/.test(content)) return true
    if (/^\|.+\|\s*$/m.test(content)) return true
    if (/^>\s/m.test(content)) return true
    if (/```/.test(content)) return true
    return false
  }

  function startPreviewPreload() {
    var boot = window.__MARKFLY_BOOT__
    var content = boot && boot[0] ? boot[0].content : ''
    if (!needsFullPreviewLite(content || '')) return
    if (window.__MARKFLY_MEDIUM_PRELOAD__) return
    var src = window.__MARKFLY_PRELOAD_ENTRY__
    if (!src) return
    if (document.querySelector('script[data-markfly-preload]')) return
    var script = document.createElement('script')
    script.type = 'module'
    script.crossOrigin = 'anonymous'
    script.src = src
    script.setAttribute('data-markfly-preload', '1')
    document.head.appendChild(script)
  }

  function deferAppLoad() {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        if (typeof requestIdleCallback === 'function') {
          requestIdleCallback(loadApp, { timeout: 400 })
        } else {
          setTimeout(loadApp, 150)
        }
      })
    })
  }

  function waitForBootInjected(then) {
    var start = Date.now()
    function poll() {
      if (window.__MARKFLY_BOOT__ && window.__MARKFLY_BOOT__.length > 0) {
        then(true)
        return
      }
      if (Date.now() - start < 600) {
        requestAnimationFrame(poll)
        return
      }
      then(false)
    }
    poll()
  }

  waitForBootInjected(function (hasBoot) {
    if (hasBoot) {
      startPreviewPreload()
      deferAppLoad()
    } else {
      loadApp()
    }
  })
})()
