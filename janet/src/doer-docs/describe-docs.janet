#
# Write doer documentation to the console. Used by `gurp describe`
# 
(use ./lib)
(use ./formatting)
(use ../command-helpers)
(import ../user-helpers :prefix "" :only [compact])
(import ../doers :prefix "")

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

(defn lay-out-help
  "Returns an array of strings. The first element has the given leader right-
  aligned, along with the first words of 'words'. The remaining elements are
  the rest of 'words', left padded so they form a neat column"
  [leader words pad-width whole-width]

  (let [pad (string/repeat " " pad-width)
        invisible (- (length leader) (length (strip-ansi leader)) 2)
        raw-words (array ;(compact (string/split " " words)) nil)
        lines @[]
        format-string (string "%" (+ pad-width invisible) "s ")]

    (var line (string/format format-string leader))

    (loop [word :in raw-words]
      (if (or (nil? word) (> (+ (length word) (length line) 2) whole-width))
        (do
          (array/push lines line)
          (set line (string pad word)))
        (set line (string line " " word))))
    lines))

(defn format-properties
  "Returns a single string of a block of properties, formatted for output."
  [props]
  (if (empty? props)
    "  None"
    (do
      (def field-width-prop-names (field-width (map bold (keys props))))
      (def stringy-types (type-list (map |(get $ :types) (values props))))
      (def field-width-prop-types (field-width stringy-types))
      (def leader-width (+ field-width-prop-names field-width-prop-types))
      (def leader-format-string
        (string "  %-" field-width-prop-names "s %-" field-width-prop-types "s"))

      (join-lines
        (flatten
          (seq [[prop-name prop-vals] :pairs props]
            (def leader
              (string/format leader-format-string
                             (bold prop-name)
                             (flatten-types (prop-vals :types))))

            (lay-out-help
              leader
              (prop-vals :help)
              (- leader-width 3)
              (term-width))))))))


(defn help-for-doer
  "Returns a multiline string showing keys supported by the given doer"
  [doer]
  (string
    (bold-underline doer)
    "\n"
    (join-lines
      (lay-out-help "" (doer-lookup doer :description) 2 (term-width)))
    "\n"
    "\n"
    (bold-underline (a/b doer "ensure"))
    "\n"
    (string "  " (bold "name")
            (if-let [name-str (doer-lookup doer :name-is)]
              (string "  [:string]  " name-str)
              "This resource does not take a name parameter"))
    "\n"
    "\n"
    (bold "Mandatory properties")
    "\n"
    (format-properties (doer-lookup doer :mandatory-props-ensure))
    "\n"
    "\n"
    (bold "Optional properties")
    "\n"
    (format-properties (doer-lookup doer :optional-props-ensure))
    "\n"
    "\n"
    (bold-underline (a/b doer "remove"))
    "\n"
    "\n"
    (bold "Mandatory properties")
    "\n"
    (format-properties (doer-lookup doer :mandatory-props-remove))
    "\n"
    "\n"
    (bold "Optional properties")
    "\n"
    (format-properties (doer-lookup doer :optional-props-remove))
    "\n"))

(defn help-for-sub-resource
  "Returns a multiline string showing keys supported by the given sub-resource"
  [doer subresource]

  (string
    (bold-underline (a/b doer subresource))
    "\n"
    (join-lines (lay-out-help
                  ""
                  (subresource-lookup doer subresource :description)
                  2
                  (term-width)))
    "\n"
    "\n"
    (string "  "
            (bold "name")
            "  [:string]  "
            (subresource-lookup doer subresource :name-is))
    "\n"
    "\n"
    (bold "Mandatory properties")
    "\n"
    (format-properties (subresource-lookup doer subresource :mandatory-props))
    "\n"
    "\n"
    (bold "Optional properties")
    "\n"
    (format-properties (subresource-lookup doer subresource :optional-props))
    "\n"))
