# Write doer documentation to the console. Used by `gurp describe`

(import ../command-lib :only [strip-ansi])
(import ../doers/lib/lib :only [comma-sep] :as doer-lib)
(import ../dsl :only [fields])
(import ../lib :only [drop-empty] :as gurp-lib)
(import ./base)
(import ./formatting :only [bold bold-underline underline] :prefix "")
(import ./lib :only [join-lines term-width])

(defn- indent [text]
  (as-> text _
        (string/split "\n" _)
        (map (partial string "  ") _)
        (string/join _ "\n")))

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
        raw-words (array ;(gurp-lib/drop-empty (dsl/fields words)) nil)
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

(def Inline
  (table/setproto
    @{:header-1 bold-underline
      :header-2 bold
      :example-formatter indent
      :format-properties format-properties
      :code (fn [code] (string "\n" (indent code)))

      :description (fn [self text]
                     (lib/join-lines
                       (lay-out-help "" text 2 (lib/term-width))))

      :note-formatter (fn [note]
                        (lib/join-lines
                          (lay-out-help "" note 2 (lib/term-width))))

      :properties (fn [self prop-type properties]
                    (string ((self :header-2)
                              (string/format "%s properties" prop-type))
                            "\n"
                            ((self :format-properties) properties)))

      :name (fn [self name-is]
              (if name-is
                (string/format "  %s  [:string]  %s" (bold "name") name-is)
                "Does not take a name parameter."))} base/Base))

(defn help-for-doer [doer]
  (def doccer (:new Inline (keyword doer)))
  (:render-doer doccer))

(defn help-for-helper [doer helper]
  (def doccer (:new Inline (keyword doer) (keyword helper)))
  (:render-helper doccer))
