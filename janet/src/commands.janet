#
# Interfaces called by the Gurp binary.
# 
(import ./formatting)
(import ./doers :prefix "")

(defn term-width
  "Gurp sets the width of the user terminal as a dyn"
  []
  (dyn :term-width 80))

(defn repo-root []
  (if-let [from-gurp (dyn :repo-root)]
    from-gurp
    (->> (dyn *current-file*)
         (os/realpath)
         (peg/replace '(* "/janet" (some 1)) ""))))

(defn doer-root []
  (string (repo-root) "/janet/src/doers"))

(defn doers []
  (seq [doer :in (os/dir (doer-root))]
    (string/replace ".janet" "" doer)))

(def doc-dir (string (repo-root) "/doc/doers"))

(defn list-doers
  "Returns a multiline string, pairing doers with their descriptions. Used by
  Gurp's 'doers' command"
  []
  (def descriptions [["apk" apk/description]
                     ["bridge" bridge/description]
                     ["cron" cron/description]
                     ["directory" directory/description]
                     ["etherstub" etherstub/description]
                     ["file-line" file-line/description]
                     ["file" file/description]
                     ["gem" gem/description]
                     ["group" group/description]
                     ["ip-address" ip-address/description]
                     ["ip-interface" ip-interface/description]
                     ["ip-properties" ip-properties/description]
                     ["ipnat" ipnat/description]
                     ["misc" misc/description]
                     ["network-flow" network-flow/description]
                     ["pkg" pkg/description]
                     ["pkgin" pkgin/description]
                     ["publisher" publisher/description]
                     ["route" route/description]
                     ["smf" smf/description]
                     ["svc" svc/description]
                     ["svcprop" svcprop/description]
                     ["symlink" symlink/description]
                     ["user" user/description]
                     ["vlan" vlan/description]
                     ["vnic" vnic/description]
                     ["zfs" zfs/description]
                     ["zone" zone/description]])

  (string/join
    (flatten
      (map |(formatting/lay-out-help ;$ 25 (term-width)) descriptions))
    "\n"))

(defn- field-width
  "Returns the width of a field which can accomodate the longest value in list"
  [list]
  (if (empty? list)
    0
    (max (splice (map length list)))))

(defn flatten-types
  [types]
  (string "[" (string/join (map |(string/format "%p" $) types) " ") "]"))

(defn type-list
  [types]
  (map flatten-types types))

(defn strip-ansi [s]
  "Remove ANSI escape codes from string"
  (string/replace-all (peg/compile ~(* "\e[" (any (if-not "m" 1)) "m")) "" s))

(defn format-properties
  "Returns a single string of a block of properties, formatted for output."
  [props]
  (if (empty? props)
    "  None"
    (do
      (def field-width-prop-names (field-width (map formatting/bold (keys props))))
      (def stringy-types (type-list (map |(get $ :types) (values props))))
      (def field-width-prop-types (field-width stringy-types))
      (def leader-width (+ field-width-prop-names field-width-prop-types))
      (def leader-format-string
        (string "  %-" field-width-prop-names "s %-" field-width-prop-types "s"))

      (string/join
        (flatten
          (seq [[prop-name prop-vals] :pairs props]
            (def leader
              (string/format leader-format-string
                             (formatting/bold prop-name)
                             (flatten-types (prop-vals :types))))

            (formatting/lay-out-help leader (prop-vals :help) (- leader-width 3) (term-width))))
        "\n"))))

(defn doer-lookup
  "Fetch the given binding from the given doer definition file"
  [doer binding]
  (try
    (do
      (def lookup (symbol (string doer "/" binding)))
      (eval lookup))
    ([_] nil)))

(defn subresource-lookup
  "As doer-lookup but for subresources"
  [doer subresource binding]
  (def lookup (symbol (string doer "/" binding "-" subresource)))
  (eval lookup))

(defn help-for-doer
  "Returns a multiline string showing keys supported by the given doer"
  [doer]
  (string
    (formatting/bold-underline doer)
    "\n"
    (string/join (formatting/lay-out-help "" (doer-lookup doer :description) 2 (term-width)) "\n")
    "\n"
    "\n"
    (formatting/bold-underline (string doer "/ensure"))
    "\n"
    (string "  " (formatting/bold "name")
            (if-let [name-str (doer-lookup doer :name-is)]
              (string "  [:string]  " name-str)
              "This resource does not take a name parameter"))
    "\n"
    "\n"
    (formatting/bold "Mandatory properties")
    "\n"
    (format-properties (doer-lookup doer :mandatory-props-ensure))
    "\n"
    "\n"
    (formatting/bold "Optional properties")
    "\n"
    (format-properties (doer-lookup doer :optional-props-ensure))
    "\n"
    "\n"
    (formatting/bold-underline (string doer "/remove"))
    "\n"
    "\n"
    (formatting/bold "Mandatory properties")
    "\n"
    (format-properties (doer-lookup doer :mandatory-props-remove))
    "\n"
    "\n"
    (formatting/bold "Optional properties")
    "\n"
    (format-properties (doer-lookup doer :optional-props-remove))
    "\n"))

(defn help-for-sub-resource
  "Returns a multiline string showing keys supported by the given sub-resource"
  [doer subresource]

  (string
    (formatting/bold-underline (string doer "/" subresource))
    "\n"
    (string/join (formatting/lay-out-help "" (subresource-lookup doer subresource :description) 2 (term-width)) "\n")
    "\n"
    "\n"
    (string "  " (formatting/bold "name") "  [:string]  " (subresource-lookup doer subresource :name-is))
    "\n"
    "\n"
    (formatting/bold "Mandatory properties")
    "\n"
    (format-properties (subresource-lookup doer subresource :mandatory-props))
    "\n"
    "\n"
    (formatting/bold "Optional properties")
    "\n"
    (format-properties (subresource-lookup doer subresource :optional-props))
    "\n"))

(defn help-for
  "Called by the Rust 'describe' command"
  [object]
  (try
    (print
      (if (string/find "/" object)
        (help-for-sub-resource (splice (string/split "/" object 0 2)))
        (help-for-doer object)))
    ([_e]
      (eprint "No help for '" object "'"))))
