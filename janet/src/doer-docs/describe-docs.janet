#
# Write doer documentation to the console. Used by `gurp describe`
# 
(use ./lib)
(use ./formatting)
(use ../command-lib)
(use ../lib)
(import ../doers/lib :prefix "" :only [comma-sep])
(use ../../test/doers/test-lib)
(import ../dsl :prefix ""
        :only [fields files-in basename lines drop-empties pathcat parent])
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
  (if (empty? list) 0 (max ;(map length list))))

(defn- flatten-types
  [types]
  (string/format "[%s]" (string/join
                          (map (partial string/format "%p") types)
                          " ")))

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
        raw-words (array ;(drop-empties (fields words)) nil)
        lines @[]
        format-string (string/format "%%%ds " (+ pad-width invisible))]

    (var line (string/format format-string leader))

    (loop [word :in raw-words]
      (if (or (nil? word)
              (< whole-width (+ (length word) (length line) 2)))
        (do
          (array/push lines line)
          (set line (string pad word)))
        (set line (string/format "%s %s" line word))))
    lines))

(defn construct-property-value
  [prop]
  (def ret @[(prop :help)])

  (when-let [allowed-values (prop :values)]
    (array/concat ret (string "Allowed values: " (comma-sep allowed-values))))

  (when-let [default-value (prop :default)]
    (array/concat ret (string/format "Default: `%p`" default-value)))

  (string (string/join ret ". ") "."))

(defn format-properties
  "Returns a single string of a block of properties, formatted for output."
  [props]
  (if (empty? props)
    "  None"
    (let [field-width-prop-names (field-width (map bold (keys props)))
          stringy-types (type-list (map |(get $ :types) (values props)))
          field-width-prop-types (field-width stringy-types)
          leader-width (+ field-width-prop-names field-width-prop-types)
          leader-format-string (string/format "  %%-%ds %%-%ds"
                                              field-width-prop-names
                                              field-width-prop-types)]

      (-> (catseq [[prop-name prop-vals] :pairs props]
            (lay-out-help (string/format leader-format-string
                                         (bold prop-name)
                                         (flatten-types (prop-vals :types)))
                          (construct-property-value prop-vals)
                          (- leader-width 3)
                          (term-width)))
          (join-lines)))))

(defn note-formatter
  [note]
  (string (join-lines (lay-out-help "" note 2 (term-width))) "\n\n"))

(defn- examples-as-map-fn
  [dir]

  (defn map-fn [file]
    (tuple (basename file) (slurp file)))

  (struct ;(mapcat map-fn (files-in dir :file))))

(def code-examples
  "This def pulls the examples into the Gurp Janet image"
  (let [doer-dirs (files-in example-root :directory)
        doers (map basename doer-dirs)
        examples (map examples-as-map-fn doer-dirs)]
    (zipcoll doers examples)))

(defn- code-example
  "Returns a string formatted for doer/action, formatted via code-bock-fn"
  [code-block-fn doer action]
  (defn example-filter
    [[k v]]
    (let [prefix (string action "-")]
      (when (string/has-prefix? prefix k) (code-block v))))

  (-> (when-let [examples (get code-examples (string doer))]
        (keep example-filter (sorted (pairs examples))))
      (string/join)))

(defn- title-block
  [title]
  (string (bold-underline title) "\n"))

(defn- description-block
  [doer &opt helper]
  (string "\n"
          (join-lines
            (lay-out-help "" (lookup doer :description helper) 2 (term-width)))
          "\n"))

(defn- name-block
  [doer &opt helper]
  (if-let [name-str (lookup doer :name-is helper)]
    (string/format "  %s  [:string]  %s\n" (bold "name") name-str)
    "Does not take a name parameter.\n"))

(defn properties-block
  [doer prop-type action]
  (let [lookup-key (keyword (string/ascii-lower prop-type) "-props-" action)
        properties (lookup doer lookup-key)]
    (string "\n" (bold (string/format "%s properties" prop-type)) "\n"
            (format-properties properties)
            "\n")))

(defn- examples-block
  [doer action-or-helper]
  (string "\n" (bold "Examples") "\n"
          (code-example code-block doer action-or-helper)))

(defn- action-block
  [doer action-or-helper &opt no-name]
  (if (lookup doer action-or-helper)
    (string "\n"
            (if-not no-name (string (bold-underline (a/b doer action-or-helper)) "\n"))
            (if (has-value? [:ensure :remove] action-or-helper)
              (name-block doer)
              (name-block doer action-or-helper))
            (properties-block doer "Mandatory" action-or-helper)
            (properties-block doer "Optional" action-or-helper)
            (examples-block doer action-or-helper))
    (string/format "There is no %s/%s action.\n" doer action-or-helper)))

(defn- notes-block
  [doer &opt helper]
  (when-let [notes (lookup doer :notes helper)]
    (string "\n" (bold "Notes") "\n" ;(map note-formatter notes))))

(defn help-for-doer
  [doer]
  (string (title-block doer)
          (description-block doer)
          (action-block doer :ensure)
          (action-block doer :remove)
          (notes-block doer)))

(defn help-for-helper
  [doer helper]
  (string (title-block (a/b doer helper))
          (description-block doer helper)
          (action-block doer helper :no-name)
          (notes-block doer helper)))
