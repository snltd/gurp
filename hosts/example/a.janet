#!/usr/bin/env janet
#
(defn with-structure [f]
  (let [pkg-list ["merp" "byerp" "gurp"]]
    (fn [& args] (apply f pkg-list args)))
  )

(var merp
  (with-structure
    (fn [struct] (pp (filter |(string/find "er" $) struct)))))


  (merp)
