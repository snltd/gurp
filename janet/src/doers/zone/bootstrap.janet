(use ../lib)

(def default-gurp-path "/var/tmp/gurp")

(defhelper :zone :bootstrap
  "Tells gurp how to bootstrap a newly created zone."
  :exactly-one-of [:file :server]

  :mandatory-props
  {:copy-self {:types [:boolean]
               :help (string/format
                       "copy the running gurp binary into the zone at `%s`"
                       default-gurp-path)
               :default true}
   :gurp-binary {:types [:string :buffer]
                 :help "path to the in-zone gurp used for bootstrapping"
                 :default default-gurp-path}}

  :optional-props
  {:server {:types [:string]
            :help "hostname/IP address of server to install from"}
   :hostname {:types [:string]
              :help "hostname of client being bootstrapped"}
   :file {:types [:string]
          :help "fully qualified path of file in zone which will be used to
                 bootstrap"}
   :debug {:types [:boolean]
           :help "run the bootstrapping gurp in debug mode"}}

  :notes
  ["On a bootstrap run, `gurp-user-defs` contains `:is-bootstrap true`, so
    you can change behaviour on an initial run."])

(make-helper-fn "bootstrap" :without-name true)
