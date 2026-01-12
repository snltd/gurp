(use ./user-helpers)

(defn compact
  "Remove empty elements from an array"
  [vector]
  (filter |(not (empty? $)) vector))

(defn qualified-path?
  "Returns true if the argument looks like a fully qualified path"
  [path]
  (string/has-prefix? "/" path))

(defn qualify-from-path
  "We expect files to be in a directory `files/` at the same level as
  the role file which references those files. This expects a path relative
  to that directory, and returns the fully qualified path, but if it gets
  a fully qualified path, it simply returns it"
  [file-name]

  (if (qualified-path? file-name)
    file-name
    (do
      (if (nil? (dyn :gurp-config-root))
        (error
          (string "cannot qualify path for " file-name ": gurp-config-root is not set")))
      (pathcat (dyn :gurp-config-root) "files" file-name))))

