(use ./lib)
(import ../collector)

(def doer :symlink)
(def description "Create and remove symbolic links.")
(def name-is "Qualified path to the link that will be created")
(def mandatory-props-ensure
  {:source {:types [:string]
            :help "The file the symlink points to"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a symlink path and target, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a symlink path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["If the :source doesn't exist, you get an error."
   "Files are ensured before links, so you can make a file and link to it."
   "If the link exists and points to the wrong file, it will be removed and
    re-created, and if it exists but is not a link, that's an error."])
