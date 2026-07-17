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
      deferAppLoad()
    } else {
      loadApp()
    }
  })
})()
