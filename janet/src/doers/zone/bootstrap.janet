(use ../lib)

(defhelper :zone :bootstrap
  "Tells gurp how to bootstrap a newly created zone."

  :optional-props
  {:server {:types [:string]
            :help "hostname/IP address of server to install from"}
   :hostname {:types [:string]
              :help "hostname of client being bootstrapped"}
   :file {:types [:string]
          :help "fully qualified path of file in zone which will be used to
                 bootstrap"}}

  :notes
  ["You must supply exactly one of `:file` and `:server`."])

(defn bootstrap
  "Given a spec, return config to bootstrap a zone"
  [& spec]
  (let [name "NO-NAME"
        spec-struct (make-spec-struct ;spec)
        expanded-spec (spec-with-defaults defaults-bootstrap spec-struct)
        spec-table (pinpoint-error :bootstrap
                                   (checked-spec expanded-spec
                                                 mandatory-props-bootstrap
                                                 optional-props-bootstrap))]

    (struct :bootstrap spec-table)))
