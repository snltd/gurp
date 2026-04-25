#
# Interfaces called by the Gurp binary.
# 
(use ./doers)
(use ./doer-docs/describe-docs)
(use ./doer-docs/formatting)
(use ./doer-docs/lib)

(def doers (doers))

(defn help-for
  "Returns a multiline string describing a doer or helpers. Called by Gurp's
  'describe' command"
  [object]
  # (try
      (if (string/find "/" object)
        (help-for-helpers (splice (string/split "/" object 0 2)))
        (help-for-doer object))
    # ([_e]
      # (string "No help for '" object "'"))))
      )

(defn list-doers
  "Returns a multiline string, pairing doers with their descriptions. Used by
  Gurp's 'doers' command"
  []
  (join-lines
    (catseq [doer :in doers]
      (lay-out-help
        (bold doer)
        (doer-lookup (keyword doer) :description)
        28
        (term-width)))))
