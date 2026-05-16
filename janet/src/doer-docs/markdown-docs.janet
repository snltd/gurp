#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests. Used by doers/build.sh, and
# not built into the Gurp lib.
# 
(use ./lib)
(use ./markdown-dsl)
(import ../dsl :prefix "" :only [pathcat])

(def doc-dir (pathcat (repo-root) "/doc/doers"))

(defn markdown-note
  [note]
  (string "- " (squeeze note) "\n"))

(defn props-to-row
  "Make an a array whose elements are cells for a row of a table"
  [property prop-vals defaults]
  [(code (string/format "%v" property))
   (code (string/join (get prop-vals :types) " "))
   (squeeze (get prop-vals :help)) # fail hard if not set
   (if-let [default-val (get defaults property)]
     (code (string/format "%m" default-val))
     "")])

(defmacro property-table
  [doer importance action]
  (with-syms [$prop-key $properties $heading $prop $vals]
    ~(do
       (let [$prop-key (keyword ,importance "-props-" ,action)
             $properties (doer-lookup ,doer $prop-key)
             $heading (h3 (title-words ,importance "properties"))]

         (if (empty? $properties)
           (string $heading "\n" "None" "\n")
           (string
             $heading
             "\n"
             (table-header :key :type :description :default)
             (string/join
               (sorted
                 (seq [[$prop $vals] :pairs $properties]
                   (table-row
                     (props-to-row
                       $prop
                       $vals
                       (doer-lookup ,doer
                                    (keyword "defaults-" ,action)))))))))))))


(defn markdown-for-doer
  "Returns a multiline string of markdown for the given doer"
  [doer]
  (string
    (h1 doer)
    "\n"
    (squeeze (doer-lookup doer :description))
    "\n"
    "\n"
    (h2 "Resource Name")
    "\n"
    (if-let [name-is (doer-lookup doer :name-is)]
      (string name-is " (`:string`)")
      "This doer does not accept a name")
    "\n"
    "\n"
    (h2 (string doer "/ensure"))
    "\n"
    (code-example code-block doer :ensure)
    (property-table doer :mandatory :ensure)
    "\n"
    (property-table doer :optional :ensure)
    "\n"

    (if (doer-lookup doer :remove)
      (string
        (h2 (string doer "/remove"))
        "\n"
        (code-example code-block doer :remove)
        (property-table doer :mandatory :remove)
        "\n"
        (property-table doer :optional :remove)
        "\n")
      (string
        (h2 (string doer "/remove"))
        "\nThere is no " doer "/remove."))

    (if-let [notes (doer-lookup doer :notes)]
      (string (h2 "Notes") "\n" (splice (map markdown-note notes))))))

(defn code-example
  [code-block-fn doer action]
  (string/join
    (filter truthy?
            (seq [file :in (sorted (os/dir (pathcat example-root doer)))]
              (when (string/has-prefix? action file)
                (code-block-fn (slurp (pathcat example-root doer file))))))))

(defn markdown-for-helpers
  "Returns a multiline string showing keys supported by the given helpers"
  [doer-dir doer helpers]
  (string
    (h1 (string doer "/" helpers))
    "\n"
    (doer-lookup doer (keyword :description- helpers))
    "\n"
    "\n"
    (h2 "Name")
    "\n"
    (if-let [name-is (doer-lookup doer (keyword :name-is- helpers))]
      (string name-is " (`:string`)")
      "This helpers does not accept a name")
    "\n"
    "\n"
    (code-example code-block doer helpers)
    (property-table doer :mandatory helpers)
    "\n"
    (property-table doer :optional helpers)
    "\n"
    (if-let [notes (helpers-lookup doer helpers :notes)]
      (string (h2 "Notes") "\n" (splice (map markdown-note notes))))))

(defn markdown-for-helperss
  [doer]
  (def doer-dir (string (doer-root) "/" doer))
  (if (os/stat doer-dir)
    (string/join
      (seq [helpers :in (sorted (os/dir doer-dir))]
        (try
          (markdown-for-helpers doer-dir doer (string/replace ".janet" "" helpers))
          ([e] (eprint "Error on " helpers ": " e))))
      "\n")))

(defn generate-docs-to-stdout
  "Used by bin/generate-docs.janet"
  [doers]
  (loop [arg :in doers]
    (print
      (markdown-for-doer arg)
      (markdown-for-helperss arg))))

(defn generate-all-docs
  "For each doer, create a doer.md file under /doc/doers. Each markdown file
  documents the core doer and any helperss"
  []
  (if-not (os/stat doc-dir)
    (os/mkdir doc-dir))

  (loop [doer :in (doers) :unless (= doer "lib")]
    (def md-file (string doc-dir "/" doer ".md"))
    (print "writing " doer " -> " md-file)
    (def fh (file/open md-file :w))
    (file/write fh
                (string
                  (markdown-for-doer (symbol doer))
                  (markdown-for-helperss (symbol doer))))))
