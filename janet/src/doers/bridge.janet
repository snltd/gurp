(use ./lib)
(import ../collector)

(def doer :bridge)
(def description "Create and modify ethernet bridges.")
(def name-is "Any valid bridge name")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:protect
   {:types [:string]
    :help "Protection method: defaults to stp"}
   :priority
   {:types [:number]
    :help "Bridge priority. 0 to 61440"}
   :max-age
   {:types [:number]
    :help "Maximum age, in seconds, for STP configuration information."}
   :hello-time
   {:types [:number]
    :help "STP hello time value, in seconds"}
   :forward-delay
   {:types [:number]
    :help "STP forward delay time, in seconds. 4 to 30"}
   :force-protocol
   {:types [:number]
    :help "MSTP forced maximum supported protocol"}
   :links
   {:types [:tuple :array]
    :help "Existing links which should be attached to the bridge"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {:priority 32768
                      :protect "stp"
                      :forward-delay 15
                      :force-protocol 3
                      :hello-time 2
                      :max-age 20})
(def defaults-remove {})

(defn ensure
  "Given a bridge name and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a bridge name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
