#
# Write doer documentation to the console. Used by `gurp describe`
# 
(use ./lib)
(use ./formatting)
(use ../command-lib)
(use ../lib)
(use ../../test/doers/test-lib)
(import ../dsl :prefix "" :only [lines compact pathcat parent])
(import ../doers :prefix "")

(defn- indent [text]
  (as-> text _
    (string/split "\n" _)
    (map (partial string "  ") _)
    (string/join _ "\n")))

(defn code-block [code]
  (string "\n" (indent code)))

(defn- field-width
  "Returns the width of a field which can accommodate the longest value in list"
  [list]
  (if (empty? list)
    0
    (max (splice (map length list)))))

(defn- flatten-types
  [types]
  (string "[" (string/join (map |(string/format "%p" $) types) " ") "]"))

(defn- type-list
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

(defn note
  [note]
  (string (join-lines (lay-out-help "" note 2 (term-width))) "\n\n"))

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
    (if-let [name-str (doer-lookup doer :name-is)]
      (string "  " (bold "name")
              "  [:string]  " name-str)
      "\nThis doer does not take a name parameter.")
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
    (if (doer-lookup doer :remove)
      (string
        (bold "Mandatory properties")
        "\n"
        (format-properties (doer-lookup doer :mandatory-props-remove))
        "\n"
        "\n"
        (bold "Optional properties")
        "\n"
        (format-properties (doer-lookup doer :optional-props-remove))
        "\n")
      (string "There is no " doer "/remove action.\n"))

    (if-let [notes (doer-lookup doer :notes)]
      (string "\n" (bold "Notes") "\n" (splice (map note notes))))

    (bold "EXAMPLES")
    "\n"
    (code-example code-block doer :ensure)
    (code-example code-block doer :remove)))

(defn help-for-helpers
  "Returns a multiline string showing keys supported by the given helpers"
  [doer helpers]

  (string
    (bold-underline (a/b doer helpers))
    "\n"
    (join-lines (lay-out-help
                  ""
                  (helpers-lookup doer helpers :description)
                  2
                  (term-width)))
    "\n"
    "\n"
    (string "  "
            (bold "name")
            "  [:string]  "
            (helpers-lookup doer helpers :name-is))
    "\n"
    "\n"
    (bold "Mandatory properties")
    "\n"
    (format-properties (helpers-lookup doer helpers :mandatory-props))
    "\n"
    "\n"
    (bold "Optional properties")
    "\n"
    (format-properties (helpers-lookup doer helpers :optional-props))
    "\n"
    (if-let [notes (helpers-lookup doer helpers :notes)]
      (string "\n" (bold "Notes") "\n" (splice (map note notes))))))
