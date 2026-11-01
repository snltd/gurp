(use ../lib/definers)
(import ../lib/lib :only [comma-sep])

(def allowed-methods ["start" "stop" "refresh" "reload"])
(def context-props [:user :group :privileges :environment])

(defn join-privs [priv-list] (string/join priv-list ","))

(defn finaliser [spec-table]
  (let [context-table @{}]
    (loop [prop :in context-props]
      (when-let [spec-value (spec-table prop)]
        (put-in context-table [prop] spec-value)
        (set (spec-table prop) nil)))

    (if-not (empty? context-table) (put-in spec-table [:context] context-table))
    spec-table))

(defhelper :smf :method
  "Defines an SMF method to launch a service state"
  :name-is (string "One of " (lib/comma-sep allowed-methods))
  :unique true
  (helper
    :finalise-with finaliser
    (property "name" [:string] :mandatory "Type of method"
              :values allowed-methods)
    (property "exec" [:string] :mandatory "Method or command to execute")
    (property "timeout" [:number] :mandatory "Seconds until method times out"
              :default 60)
    (property "user" [:string] :optional "User the method runs as")
    (property "group" [:string] :optional "Group the method runs as")
    (property "privileges" [:tuple] :optional
              "Privileges the method has. Use ! to remove them"
              :transform-with join-privs)
    (property "environment" [:struct] :optional
              "Environment variables set inside context"))

  (notes
    "If you don't supply a `:stop-method` you get a standard `:kill` that times
     out after ten seconds. Start timeouts default to 60 seconds."))

(make-helper-fn :method :prefix-name-with-prop :name)
