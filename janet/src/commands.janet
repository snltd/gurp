# Interfaces called by the Gurp binary.

(use ./doer-docs/formatting)
(import ./facts :only [fact->fn])
(import ./doer-docs/describe-docs :only [help-for-doer help-for-helper
                                         lay-out-help])
(import ./doer-docs/lib :only [doers join-lines lookup term-width] :as docs-lib)
(import ./control-data :only [description])

(def doers (docs-lib/doers))

# Returns a multiline string describing a doer or helper. Called by Gurp's
# 'describe' command
(defn help-for
  [object]
  (try
    (if (string/find "/" object)
      (let [[doer action] (string/split "/" object 0 2)]
        (if (has-value? ["ensure" "remove"] action)
          (describe-docs/help-for-doer doer)
          (describe-docs/help-for-helper doer action)))
      (if (= object "control-data")
        (control-data/description)
        (describe-docs/help-for-doer object)))
  ([_e]
    (string "No help for '" object "'"))))

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
  (let [sorted-fact-names (sorted (keys facts/fact->fn))]
    (-> (catseq [name :in sorted-fact-names]
          (let [fact (facts/fact->fn name)]
            (describe-docs/lay-out-help (bold (string ":" name))
                                        (fact :description)
                                        28
                                        (docs-lib/term-width))))
        (docs-lib/join-lines))))
