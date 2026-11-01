#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests. Used by doers/build.sh, and
# not built into the Gurp lib.
# 
(use ./lib)
(use ./markdown-dsl)
(use ../../test/doers/lib/test-lib)
(use ../facts)

(import ../dsl :prefix "" :only [pathcat])
(import ../doers/lib :prefix "" :only [comma-sep])

(def doc-dir (pathcat (repo-root) "doc"))
(def doer-doc-dir (pathcat doc-dir "doers"))

(defn- note-formatter
  [note]
  (string "- " (squeeze note) "\n"))

(defn- examples-block
  [code-block-fn doer action]
  (let [example-files (os/dir (pathcat example-root doer))
        relevant-files (filter |(string/has-prefix? action $) example-files)]

    (if-not (empty? relevant-files)
      (string "\n" (h3 "Examples") "\n"
              ;(seq [file :in (sorted relevant-files)]
                 (code-block-fn (slurp (pathcat example-root doer file))))))))

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
   (squeeze (property-description spec))
   (if-let [default-val (spec :default)] (code default-val) "")])

(defn- property-table
  [props]
  (let [rows (seq [[prop spec] :pairs props]
               (table-row ;(props-to-row prop spec)))]
    (string (table-header :key :type :description :default)
            ;(sorted rows))))

(defn- title-block
  [title]
  (string (h1 title) "\n"))

(defn- description-block
  [doer &opt helper]
  (string (squeeze (lookup doer :description)) "\n\n"))

(defn- name-block
  [doer &opt helper]
  (string (h3 "Resource Name") "\n"
          (if-let [name-is (lookup doer :name-is helper)]
            (string name-is " (`:string`)\n")
            "Does not take a name parameter.\n")))

(defn- properties-block
  [doer prop-type action]
  (let [lookup-key (keyword (string/ascii-lower prop-type) "-props-" action)
        properties (lookup doer lookup-key)]
    (string "\n" (h3 (string/format "%s properties" prop-type)) "\n"
            (if (empty? properties)
              "None\n"
              (property-table properties)))))

(defn- notes-block
  [doer &opt helper]
  (when-let [notes (lookup doer :notes helper)]
    (string (h2 "Notes") "\n" ;(map note-formatter notes))))

(defn- action-block
  [doer action-or-helper &opt no-name]
  (if (lookup doer action-or-helper)
    (string
      (if-not no-name (string (h2 (a/b doer action-or-helper)) "\n"))
      (if (has-value? [:ensure :remove] action-or-helper)
        (name-block doer)
        (name-block doer action-or-helper))
      (properties-block doer "Mandatory" action-or-helper)
      (properties-block doer "Optional" action-or-helper)
      (examples-block code-block doer action-or-helper))
    (string/format "There is no %s/%s action.\n" doer action-or-helper)))

(defn generate-facts-doc
  "Fills in a block of doc/facts.md"
  []
  (def marker "## Full List of Facts")
  (def facts-file (pathcat doc-dir "facts.md"))

  (if-not (os/stat facts-file)
    (errorf "no facts doc at %s" facts-file))

  (let [orig (slurp facts-file)
        marker-index (string/find marker orig)
        sorted-fact-names (sorted (keys fact->fn))
        frontspiece (string/slice orig 0 (+ marker-index (length marker) 1))
        facts-list (join-lines
                     (catseq [name :in sorted-fact-names]
                       (let [fact (fact->fn name)]
                         (string/format "- %s (%s) %s"
                                        (code (keyword name))
                                        (fact :type)
                                        (squeeze (fact :description))))))]

    (print "writing facts doc -> " facts-file)
    (def fh (file/open facts-file :w))
    (file/write fh
                (string frontspiece facts-list))))

(defn markdown-for-doer
  [doer]
  (string (title-block doer)
          (description-block doer)
          (action-block doer :ensure)
          (action-block doer :remove)
          (notes-block doer)))

(defn markdown-for-helper
  [doer helper]
  (string (title-block (a/b doer helper))
          (description-block doer helper)
          (action-block doer helper :no-name)
          (notes-block doer helper)))

(defn generate-doc
  [doer-or-helper]
  (if (string/find "/" doer-or-helper)
    (markdown-for-helper ;(string/split "/" doer-or-helper))
    (markdown-for-doer doer-or-helper)))

(defn generate-docs-to-stdout
  "Used by bin/generate-docs.janet"
  [args]
  (loop [doer-or-helper :in args]
    (print (generate-doc doer-or-helper))))

(defn generate-all-docs
  "For each doer, create a doer.md file under /doc/doers. Each markdown file
  documents the core doer and any helperss"
  []
  (generate-facts-doc)

  (if-not (os/stat doer-doc-dir)
    (os/mkdir doer-doc-dir))

  (loop [doer :in (doers) :unless (= doer "lib")]
    (def md-file (string doer-doc-dir "/" doer ".md"))
    (print "writing " doer " -> " md-file)
    (def fh (file/open md-file :w))
    (file/write fh (generate-doc doer))))
