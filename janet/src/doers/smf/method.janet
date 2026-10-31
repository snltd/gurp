(use ../definers)
(import ../lib :only [comma-sep])

(def allowed-methods ["start" "stop" "refresh" "reload"])
(def context-props [:user :group :privileges :environment])

(defn join-privs [priv-list] (string/join priv-list ","))

(defhelper :smf :method
  "Defines an SMF method to launch a service state"
  :name-is (string "One of " (lib/comma-sep allowed-methods))
  :unique true
  (helper
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
# (defn method
#   "Produce an SMF exec_method, with a context"
#   [name & spec]

#   (let [doer "smf"]
#     (pinpoint-error
#       :method

#       (if-not (has-value? allowed-methods name)
#         (errorf "smf/method name must be one of %s" (lib/comma-sep allowed-methods)))

#       # We have to move context related properties (context-props) into a
#       # :context struct

#       (let [user-spec (make-spec-struct ;spec)
#             expanded-spec (spec-with-defaults
#                             (extract-defaults mandatory-props-method)
#                             user-spec)
#             spec-table (checked-spec expanded-spec
#                                      mandatory-props-method
#                                      optional-props-method)
#             context-table @{}]

#         (loop [prop :in context-props]
#           (when-let [spec-value (get spec-table prop)]
#             (def value-to-move (if (= prop :privileges)
#                                  (string/join spec-value ",")
#                                  spec-value))

#             (set (context-table prop) value-to-move)
#             (set (spec-table prop) nil)))

#         (if-not (empty? context-table)
#           (set (spec-table :context) (table/to-struct context-table)))

#         (struct (keyword (string name "-method")) spec-table)))))

