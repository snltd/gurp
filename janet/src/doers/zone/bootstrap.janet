(use ../lib)

(def default-gurp-path "/var/tmp/gurp")

(defhelper :zone :bootstrap
  "Tells gurp how to bootstrap a newly created zone."

  :optional-props
  {:server {:types [:string]
            :help "hostname/IP address of server to install from"}
   :hostname {:types [:string]
              :help "hostname of client being bootstrapped"}
   :file {:types [:string]
          :help "fully qualified path of file in zone which will be used to
                 bootstrap"}
   :copy-self {:types [:boolean]
               :help (string/format
                       "copy the running gurp binary into the zone
                                    at `%s`"
                       default-gurp-path)}
   :gurp-binary {:types [:string :buffer]
                 :help (string/format
                         "path to the in-zone gurp used for bootstrapping"
                         default-gurp-path)}
   :debug {:types [:boolean]
           :help "run the bootstrapping gurp in debug mode"}}

  :defaults
  {:copy-self true
   :debug false
   :gurp-binary default-gurp-path}

  :notes
  ["You must supply exactly one of `:file` and `:server`."
   "On a bootstrap run, `gurp-user-defs` contains `:is-bootstrap true`, so
    you can change behaviour on an initial run."])


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
