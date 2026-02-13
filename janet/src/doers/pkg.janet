(use ./lib)
(import ../collector)

(def doer :pkg)
(def description "Install and uninstall pkg(5) packages.")
(def name-is "Package name, of the form ooce/editor/helix")
(def mandatory-props-ensure {})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["You specify packages by name, so `ooce/editor/helix` rather than
    `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means
    you can't request specific versions. It isn't remotely smart and can't
    understand that `helix` is `ooce/editor/helix`."
   "Gurp does not support package mediators."
   "If Gurp runs with `--noop`, `pkg` will be executed with the `-n` flag.
    Therefore it can cause a noop run to fail."
   "These limitations mean we can ensure all packages are installed with just
    two calls to `pkg(1)`. This is a huge speed win over dealing with each package
    individually"])
