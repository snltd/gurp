## Write doer documentation to the console. Used by `gurp describe`
# 
(import ../../test/doers/lib/test-lib :only [example-root])
(import ../command-lib :only [strip-ansi])
(import ../doers/lib/lib :only [comma-sep] :as doer-lib)
(import ../dsl :only [basename fields files-in])
(import ../lib :only [drop-empties ] :as gurp-lib)
(import ./formatting :only [bold bold-underline underline] :prefix "")
(import ./lib :only [a/b join-lines joint lookup props-without-name term-width])

(defn- indent [text]
  (as-> text _
        (string/split "\n" _)
        (map (partial string "  ") _)
        (string/join _ "\n")))

(defn code-block [code]
  (string "\n" (indent code)))

# Returns the width of a field which can accommodate the longest value in list
(defn- field-width [list]
  (if (empty? list) 0 (max ;(map length list))))

(defn- flatten-types [types]
  (string/format "[%s]" (string/join (map (partial string/format "%p") types) " ")))

(defn- type-list [types]
  (map flatten-types types))

# Returns an array of strings. The first element has the given leader right-
# aligned, along with the first words of 'words'. The remaining elements are
# the rest of 'words', left padded so they form a neat column
(defn lay-out-help [leader words pad-width whole-width]
  (let [pad (string/repeat " " pad-width)
        invisible (- (length leader) (length (command-lib/strip-ansi leader)) 2)
        raw-words (array ;(gurp-lib/drop-empties (dsl/fields words)) nil)
        lines @[]
        format-string (string/format "%%%ds " (+ pad-width invisible))]

    (var line (string/format format-string leader))

    (loop [word :in raw-words]
      (if (or (nil? word) (< whole-width (+ (length word) (length line) 2)))
        (do
          (array/push lines line)
          (set line (string pad word)))
        (set line (string/format "%s %s" line word))))
    lines))

(defn construct-property-value
  [prop]
  (let [ret @[(prop :help)]]
    (when-let [allowed-values (prop :values)]
      (array/concat ret (string "Allowed values: "
                                (doer-lib/comma-sep allowed-values))))
    (when-let [default-value (prop :default)]
      (array/concat ret (string/format "Default: `%p`" default-value)))
    (string (string/join ret ". ") ".")))

# Returns a single string of a block of properties, formatted for output.
(defn format-properties [props]
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
                          (lib/term-width)))
          (lib/join-lines)))))

(defn- note-formatter [note]
  (lib/join-lines (lay-out-help "" note 2 (lib/term-width))))

(defn- code-examples-map-fn [dir]
  (defn map-fn [file] (tuple (dsl/basename file) (slurp file)))
  (struct ;(mapcat map-fn (dsl/files-in dir :file))))

# This def is important because it pulls the examples into the Gurp Janet image
(def code-examples
  (let [doer-dirs (dsl/files-in test-lib/example-root :directory)
        doers (map dsl/basename doer-dirs)
        examples (map code-examples-map-fn doer-dirs)]
    (zipcoll doers examples)))

# Returns a string formatted for doer/action, formatted via code-bock-fn
(defn- code-example [code-block-fn doer action]
  (defn filter-fn [[k v]]
    (let [prefix (string action "-")]
      (when (string/has-prefix? prefix k) (code-block v))))

  (-> (when-let [examples (get code-examples (string doer))]
        (keep filter-fn (sorted (pairs examples))))
      (string/join)))

(defn- name-block [info]
  (if-let [name-str (info :name-is)]
    (string/format "  %s  [:string]  %s" (bold "name") name-str)
    "Does not take a name parameter."))

(defn- examples-block [doer prefixes]
  (string (bold "Examples") "\n"
          (lib/joint
            ;(catseq [prefix :in prefixes]
               (code-example code-block doer prefix)))))


(defn- properties-block [prop-type properties]
  (string (bold (string/format "%s properties" prop-type)) "\n"
          (format-properties properties)))

(defn- props-block [action-info]
  (lib/joint
    (properties-block "Mandatory" (lib/props-without-name action-info :mandatory))
    (properties-block "Optional" (lib/props-without-name action-info :optional))))

(defn- notes-block [info]
  (when-let [notes (info :notes)]
    (lib/joint (bold "Notes") ;(map note-formatter notes))))

(defn- action-block [info doer action]
  (if-let [action-info (info action)]
    (lib/joint (bold-underline (lib/a/b doer action))
               (props-block action-info))
    (string/format "There is no %s/%s action." doer action)))

(defn- title-block [& bits]
  (bold-underline (lib/a/b ;bits)))

(defn- description-block [info]
  (lib/join-lines (lay-out-help "" (info :description) 2 (lib/term-width))))

(defn help-for-doer [doer]
  (let [info (lib/lookup doer :attrs)]
    (lib/joint (title-block doer)
               (description-block info)
               (name-block info)
               (action-block info doer :ensure)
               (action-block info doer :remove)
               (notes-block info)
               (examples-block doer [:ensure :remove]))))

(defn help-for-helper [doer helper]
  (let [info (lib/lookup doer (symbol :attrs- helper))]
    (lib/joint (title-block doer helper)
               (description-block info)
               (name-block info)
               (props-block (info :helper))
               (notes-block info)
               (examples-block doer [helper]))))
