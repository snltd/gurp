# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests. Used by doers/build.sh, and
# not built into the Gurp lib.
# 
(import ./lib :only [a/b doers join-lines joint lookup props-without-name repo-root squeeze])
(import ../../test/doers/lib/test-lib :only [example-root])
(import ../facts :only [fact->fn])
(import ../dsl :only [basename files-in pathcat])
(import ../doers/lib/lib :prefix "" :only [comma-sep])
(use ./markdown-dsl)

(def doc-dir (dsl/pathcat (lib/repo-root) "doc"))
(def doer-doc-dir (dsl/pathcat doc-dir "doers"))

(defn- code-examples-map-fn [dir]
  (defn map-fn [file] (tuple (dsl/basename file) (slurp file)))
  (struct ;(mapcat map-fn (dsl/files-in dir :file))))

# This def is important because it pulls the examples into the Gurp Janet image
(def code-examples
  (let [doer-dirs (dsl/files-in test-lib/example-root :directory)
        doers (map dsl/basename doer-dirs)
        examples (map code-examples-map-fn doer-dirs)]
    (zipcoll doers examples)))

(defn- code-example [code-block-fn doer action]
  (defn filter-fn [[k v]]
    (let [prefix (string action "-")]
      (when (string/has-prefix? prefix k) (code-block v))))

  (-> (when-let [examples (get code-examples (string doer))]
        (keep filter-fn (sorted (pairs examples))))
      (string/join)))

(defn- note-formatter [note]
  (string "- " (lib/squeeze note) "\n"))

(defn- examples-block [doer prefixes]
  (string (h3 "Examples") "\n"
          (lib/joint
            ;(catseq [prefix :in prefixes]
               (code-example code-block doer prefix)))))

# (defn- examples-block
#   [code-block-fn doer action]
#   (let [example-files (os/dir (pathcat test-lib/example-root doer))
#         relevant-files (filter |(string/has-prefix? action $) example-files)]

#     (if-not (empty? relevant-files)
#       (string "\n" (h3 "Examples") "\n"
#               ;(seq [file :in (sorted relevant-files)]
#                  (code-block-fn (slurp (pathcat test-lib/example-root doer file))))))))

(defn- property-description
  [spec]
  (def ret @[(spec :help)])

  (when-let [allowed-values (spec :values)]
    (array/concat ret (string "Allowed values: " (comma-sep allowed-values))))

  (string (string/join ret ". ") "."))

(defn- props-to-row
  "Make an array whose elements are cells for a row of a table"
  [prop spec]
  [(code (string/format "%v" prop))
   (code (string/join (spec :types) " "))
   (lib/squeeze (property-description spec))
   (if-let [default-val (spec :default)] (code default-val) "")])

(defn- property-table
  [props]
  (let [rows (seq [[prop spec] :pairs props]
               (table-row ;(props-to-row prop spec)))]
    [(table-header :key :type :description :default) ;(sorted rows)]))

(defn- title-block [& args]
  (h1 (lib/a/b ;args)))

(defn- description-block [info]
  (string (lib/squeeze (info :description))))

(defn- name-block [info]
  (lib/join-lines [(h3 "Resource Name")
                   (if-let [name-is (info :name-is)]
                     (string name-is " [`:string`]")
                     "Does not take a name parameter.")]))

(defn- properties-block
  [prop-type properties]
  (lib/join-lines [(h3 (string/format "%s properties" prop-type))
                   ;(if (empty? properties)
                      ["None"]
                      (property-table properties))]))

(defn- notes-block
  [info]
  (when-let [notes (info :notes)]
    (string (h2 "Notes") "\n" ;(map note-formatter notes))))

(defn- props-block [action-info]
  (lib/joint
    (properties-block "Mandatory" (lib/props-without-name action-info :mandatory))
    (properties-block "Optional" (lib/props-without-name action-info :optional))))

(defn- action-block [info doer action]
  (if-let [action-info (info action)]
    (lib/joint (h2 (lib/a/b doer action))
               (props-block action-info))
    (string/format "There is no %s/%s action." doer action)))

(defn generate-facts-doc
  "Fills in a block of doc/facts.md"
  []
  (def marker "## Full List of Facts")
  (def facts-file (dsl/pathcat doc-dir "facts.md"))

  (if-not (os/stat facts-file)
    (errorf "no facts doc at %s" facts-file))

  (let [orig (slurp facts-file)
        marker-index (string/find marker orig)
        sorted-fact-names (sorted (keys facts/fact->fn))
        frontspiece (string/slice orig 0 (+ marker-index (length marker) 1))
        facts-list (lib/join-lines
                     (catseq [name :in sorted-fact-names]
                       (let [fact (facts/fact->fn name)]
                         (string/format "- %s (%s) %s"
                                        (code (keyword name))
                                        (fact :type)
                                        (lib/squeeze (fact :description))))))]

    (print "writing facts doc -> " facts-file)
    (def fh (file/open facts-file :w))
    (file/write fh
                (string frontspiece facts-list))))

(defn markdown-for-doer [doer]
  (let [info (lib/lookup doer :attrs)]
    (lib/joint (title-block doer)
               (description-block info)
               (name-block info)
               (action-block info doer :ensure)
               (action-block info doer :remove)
               (notes-block info)
               (examples-block doer [:ensure :remove]))))

(defn markdown-for-helper [doer helper]
  (let [info (lib/lookup doer (symbol :attrs- helper))]
    (lib/joint (title-block doer helper)
               (description-block info)
               (name-block info)
               (props-block (info :helper))
               (notes-block info)
               (examples-block doer [helper]))))

(defn generate-doc [doer-or-helper]
  (if (string/find "/" doer-or-helper)
    (markdown-for-helper ;(string/split "/" doer-or-helper))
    (markdown-for-doer doer-or-helper)))

# Used by bin/generate-docs.janet
(defn generate-docs-to-stdout [args]
  (loop [doer-or-helper :in args]
    (print (generate-doc doer-or-helper))))

# For each doer, create a doer.md file under /doc/doers. Each markdown file
# documents the core doer and any helpers
(defn generate-all-docs []
  (generate-facts-doc)

  (if-not (os/stat doer-doc-dir)
    (os/mkdir doer-doc-dir))

  (loop [doer :in (lib/doers) :unless (= doer "lib")]
    (def md-file (string doer-doc-dir "/" doer ".md"))
    (print "writing " doer " -> " md-file)
    (def fh (file/open md-file :w))
    (file/write fh (generate-doc doer))))
