# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests. Used by doers/build.sh, and
# not built into the Gurp lib.

(import ../doers/lib/lib :prefix "" :only [comma-sep] :as doer-lib)
(import ../dsl :only [pathcat])
(import ../facts :only [fact->fn])
(import ./base :only [Base])
(import ./lib :only [doers join-lines repo-root squeeze])
(use ./markdown-dsl)

(def doc-dir (dsl/pathcat (lib/repo-root) "doc"))
(def doer-doc-dir (dsl/pathcat doc-dir "doers"))

(defn- property-description [spec]
  (let [ret @[(spec :help)]]
    (when-let [allowed-values (spec :values)]
      (array/concat ret (string "Allowed values: "
                                (doer-lib/comma-sep allowed-values))))
    (string (string/join ret ". ") ".")))

# Make an array whose elements are cells for a row of a table
(defn- props-to-row [prop spec]
  [(code (string/format "%v" prop))
   (code (string/join (spec :types) " "))
   (lib/squeeze (property-description spec))
   (if-let [default-val (spec :default)]
     (code (string/format "%p" default-val)) "")])

(defn- property-table [props]
  (let [rows (seq [[prop spec] :pairs props]
               (table-row ;(props-to-row prop spec)))]
    [(table-header :key :type :description :default) ;(sorted rows)]))

# Fills in a block of doc/facts.md
(defn generate-facts-doc []
  (def marker (h2 "Full List of Facts"))
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
    (with [fh (file/open facts-file :w)]
      (file/write fh (string frontspiece facts-list)))))

(def Markdown
  (table/setproto
    @{:header-1 h1
      :header-2 h2
      :description (fn [self text] (string (lib/squeeze text)))
      :note-formatter (fn [note] (string "- " (lib/squeeze note)))
      :example-formatter code-block

      :name (fn [self name-is]
              (lib/join-lines [(h3 "Resource Name")
                               (if :name-is
                                 (string name-is " [`:string`]")
                                 "Does not take a name parameter.")]))

      :properties (fn [self prop-type properties]
                    (lib/join-lines [(h3 (string/format "%s Properties" prop-type))
                                     ;(if (empty? properties)
                                        ["None"]
                                        (property-table properties))]))} base/Base))

(defn markdown-for-doer [doer]
  (def doccer (:new Markdown (keyword doer)))
  (:render-doer doccer))

(defn markdown-for-helper [doer helper]
  (def doccer (:new Markdown (keyword doer) (keyword helper)))
  (:render-helper doccer))

(defn generate-doc [doer-or-helper]
  (if (string/find "/" doer-or-helper)
    (markdown-for-helper ;(string/split "/" doer-or-helper))
    (markdown-for-doer doer-or-helper)))

(defn generate-docs-to-stdout [args]
  (loop [doer-or-helper :in args]
    (print (generate-doc doer-or-helper))))

# For each doer, create a doer.md file under /doc/doers. Each markdown file
# documents the core doer and any helpers
(defn generate-all-docs []
  (generate-facts-doc)
  (if-not (os/stat doer-doc-dir)
    (os/mkdir doer-doc-dir))

  (loop [doer :in (lib/doers)]
    (def md-file (string doer-doc-dir "/" doer ".md"))
    (print "writing " doer " -> " md-file)
    (with [fh (file/open md-file :w)]
      (file/write fh (generate-doc doer)))))
