# Interfaces called by the Gurp binary.
# 
(use ./doers)
# (use ./doer-docs/describe-docs)
(import ./doer-docs/describe-docs :only [help-for-doer help-for-helper lay-out-help])
(use ./doer-docs/formatting)
(use ./facts)
(import ./doer-docs/lib :only [doers join-lines lookup term-width] :as docs-lib)
(import ./control-data :only [describe])

(def doers (docs-lib/doers))

# Returns a multiline string describing a doer or helper. Called by Gurp's
# 'describe' command
(defn help-for [object]
  (if (= object "control-data")
    (control-data/describe)
    (try
      (if (string/find "/" object)
        (describe-docs/help-for-helper (splice (string/split "/" object 0 2)))
        (describe-docs/help-for-doer object))
      ([_e]
        (string "No help for '" object "'")))))

# Returns a multiline string, pairing doers with their descriptions. Used by
# Gurp's 'doers' command
(defn list-doers []
  (-> (catseq [doer :in doers]
        (describe-docs/lay-out-help (bold doer)
                      ((docs-lib/lookup doer :attrs) :description)
                      28
                      (docs-lib/term-width)))
      (docs-lib/join-lines)))

# Returns a multiline string, pairing facts with their descriptions. Used by
# Gurp's 'facts' command
(defn list-facts []
  (let [sorted-fact-names (sorted (keys fact->fn))]
    (-> (catseq [name :in sorted-fact-names]
          (let [fact (fact->fn name)]
            (describe-docs/lay-out-help
              (bold (string ":" name))
              (fact :description)
              28
              (docs-lib/term-width))))
        (docs-lib/join-lines))))
