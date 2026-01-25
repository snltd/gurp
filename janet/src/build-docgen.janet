# 
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests.
# 
(if (dyn :running-embedded)
  (do
    (use doers)
    (use commands))
  (do
    (use ./doers)
    (use ./commands)))

(defn generate-docs-to-stdout
  [doers]
  (loop [arg :in doers]
    (print
      (markdown-for-doer arg)
      (markdown-for-sub-resources arg))))

(defn generate-all-docs []
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
