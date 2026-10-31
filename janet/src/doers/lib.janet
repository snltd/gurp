(import ../collector)

(defn comma-sep
  "Return a comma-separated string of the items in list. Everything is literal,
  so keywords stay keywords, and quoted strings keep their quotes."
  [list]
  (string/join (map |(string/format "%p" $) list) ", "))

(defn keys-in-struct
  "Returns those elements of key-list which are keys in struct-to-check"
  [key-list struct-to-check]
  (filter |(has-value? key-list $) (keys struct-to-check)))

(defn expand-svc-property
  "Turns the keys of a svcprop struct into structs describing a typed value"
  [value-struct]
  (tabseq [[prop-name value] :pairs value-struct]
    prop-name (match (type value)
                :keyword value
                :number {:type "integer" :value value}
                :boolean {:type "boolean" :value value}
                _ {:type "astring" :value value})))

(defn safe-val
  "Turn a string into something safe to put into a zone attr."
  [val]
  (let [contains-peg (peg/compile ~(any (+ (* ($) :W) 1)))
        escape-peg (peg/compile ~{:special (+ (/ (set "\"") "\\\"")
                                              (/ (set "\\") "\\\\")
                                              (/ (set "\n") "\\n")
                                              (/ (set "\t") "\\t")
                                              (/ (set "\r") "\\r"))
                                  :normal (if-not :special (capture 1))
                                  :main (any (+ :special :normal))})]

    (if (empty? (peg/match contains-peg val))
      val
      (string "\"" (string/join (peg/match escape-peg val) "") "\""))))
