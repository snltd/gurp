(use ./lib)
(import ../collector)

(def doer :gem)
(def description "Install and uninstall Ruby gems.")
(def name-is "Gem name")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:gem-path {:types [:string]
              :help "Path to gem executable other than /opt/ooce/bin/gem"}
   :source {:types [:string]
            :help "Source other than RubyGems. Can contain tokens and usernames"}
   :version {:types [:string]
             :help "Gem version"}})
(def mandatory-props-remove {})
(def optional-props-remove
  {:gem-path: {:types [:string]
               :help "Path to gem executable other than /opt/ooce/bin/gem"}
   :version {:types [:string]
             :help "Gem version"}})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a gem name and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a gem name and spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
