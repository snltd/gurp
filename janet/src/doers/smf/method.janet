(use ../lib)

(def allowed-methods ["start" "stop" "refresh" "reload"])
(def context-props [:user :group :privileges :environment])

(def description-method "Defines an SMF method to launch a service state")
(def name-is-method (string "One of " (comma-sep allowed-methods)))

(def optional-props-method
  {:user {:types [:string]
          :help "User the method runs as"}
   :group {:types [:string]
           :help "Group the method runs as"}
   :privileges {:types [:tuple]
                :help "Privileges the method has. Use ! to remove them"}
   :environment {:types [:struct :table]
                 :help "Environment variables set inside context"}})
(def mandatory-props-method
  {:exec
   {:types [:string]
    :help "Method or command to execute"}
   :timeout
   {:types [:number]
    :help "Seconds until method times out"}})

(def defaults-method {:timeout 60})

(defn method
  "Produce an SMF exec_method, with a context"
  [name & spec]

  (let [doer "smf"]
    (pinpoint-error
      :method

      (if-not (has-value? allowed-methods name)
        (errorf "smf/method name must be one of %s" (comma-sep allowed-methods)))

      # We have to move context related properties (context-props) into a
      # :context struct

      (let [user-spec (make-spec-struct ;spec)
            expanded-spec (spec-with-defaults defaults-method user-spec)
            spec-table (checked-spec expanded-spec
                                     mandatory-props-method
                                     optional-props-method)
            context-table @{}]

        (loop [prop :in context-props]
          (when-let [spec-value (get spec-table prop)]
            (def value-to-move (if (= prop :privileges)
                                 (string/join spec-value ",")
                                 spec-value))

            (set (context-table prop) value-to-move)
            (set (spec-table prop) nil)))

        (if-not (empty? context-table)
          (set (spec-table :context) (table/to-struct context-table)))

        (struct (keyword (string name "-method")) spec-table)))))

(def notes-method
  ["If you don't supply a `:stop-method` you get a standard `:kill` that times
    out after ten seconds. Start timeouts default to 60 seconds."])
