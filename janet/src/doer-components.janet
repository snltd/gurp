(defn checksum []
  {:types [:struct :table]
   :help "Define a checksum for a from-type resource"
   :definition
   {:type {:types [:string]
           :help "Type of checksum resource"
           :values ["suffix" "literal" "url"]}
    :value {:types [:string]
            :help "Literal value, suffix, or url"}}})
