(use ../lib)

(def description-bootstrap "Tells gurp how to bootstrap a newly created zone.")
(def name-is-bootstrap "This resource type does not accept a name")
(def mandatory-props-bootstrap {})
(def optional-props-bootstrap
  {:server {:types [:string]
            :help "hostname/IP address of server to install from"}
   :hostname {:types [:string]
              :help "hostname of client being bootstrapped"}
   :file {:types [:string]
          :help "fully qualified path of file in zone which will be used to bootstrap"}})
(def defaults-bootstrap {})

(defn bootstrap
  "Given a spec, return config to bootstrap a zone"
  [& spec]
  (def spec-struct (make-spec-struct ;spec))
  (def spec-table (checked-spec (spec-with-defaults defaults-bootstrap spec-struct)
                                mandatory-props-bootstrap
                                optional-props-bootstrap))
  (struct :bootstrap spec-table))
