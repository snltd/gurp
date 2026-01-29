#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests. Used by doers/build.sh, and
# not built into the Gurp lib.
# 
(use ./lib)
(use ./markdown-helpers)
(use ../../test/doers/_helpers)
(import ../user-helpers :prefix "" :only [pathcat])

(def doc-dir (pathcat (repo-root) "/doc/doers"))

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

(defn code-example
  [doer action]
  (string/join
    (filter truthy?
            (seq [file :in (sorted (os/dir (pathcat example-root doer)))]
              (when (string/has-prefix? action file)
                (code-block (slurp (pathcat example-root doer file))))))))

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
      "This resource does not accept a name")
    "\n"
    "\n"
    (h2 (string doer "/ensure"))
    "\n"
    (code-example doer :ensure)
    (property-table doer :mandatory :ensure)
    "\n"
    (property-table doer :optional :ensure)
    "\n"

    (if (doer-lookup doer :remove)
      (string
        (h2 (string doer "/remove"))
        "\n"
        (code-example doer :remove)
        (property-table doer :mandatory :remove)
        "\n"
        (property-table doer :optional :remove)
        "\n")
      (string
        (h2 (string doer "/remove"))
        "\nThere is no " doer "/remove."))))

(defn markdown-for-sub-resource
  "Returns a multiline string showing keys supported by the given sub-resource"
  [doer-dir doer sub-resource]
  (string
    (h1 (string doer "/" sub-resource))
    "\n"
    (doer-lookup doer (keyword :description- sub-resource))
    "\n"
    "\n"
    (h2 "Sub-Resource Name")
    "\n"
    (if-let [name-is (doer-lookup doer (keyword :name-is- sub-resource))]
      (string name-is " (`:string`)")
      "This sub-resource does not accept a name")
    "\n"
    "\n"
    (code-example doer sub-resource)
    (property-table doer :mandatory sub-resource)
    "\n"
    (property-table doer :optional sub-resource)
    "\n"))

(defn markdown-for-sub-resources
  [doer]
  (def doer-dir (string (doer-root) "/" doer))
  (if (os/stat doer-dir)
    (string/join
      (seq [sub-resource :in (sorted (os/dir doer-dir))]
        (try
          (markdown-for-sub-resource doer-dir doer (string/replace ".janet" "" sub-resource))
          ([e] (eprint "Error on " sub-resource ": " e))))
      "\n")))

(defn generate-docs-to-stdout
  "Used by bin/generate-docs.janet"
  [doers]
  (loop [arg :in doers]
    (print
      (markdown-for-doer arg)
      (markdown-for-sub-resources arg))))

(defn generate-all-docs
  "For each doer, create a doer.md file under /doc/doers. Each markdown file"
  "documents the core doer and any sub-resource helpers"
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
                  (markdown-for-sub-resources (symbol doer))))))
