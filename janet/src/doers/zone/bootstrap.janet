(import ../../collector)
(use ../definers)

(def default-gurp-path "/var/tmp/gurp")

(defhelper :zone :bootstrap
  "Tells gurp how to bootstrap a newly created zone."
  :unique true
  (helper
    :exactly-one-of [:file :server]
    (property "copy-self" [:boolean] :mandatory
              (string/format
                "copy the running gurp binary into the zone at `%s`"
                default-gurp-path)
              :default true)
    (property "gurp-binary" [:string :buffer] :mandatory
              "path to the in-zone gurp used for bootstrapping"
              :default default-gurp-path)
    (property "server" [:string] :optional
              "hostname/IP address of server to install from")
    (property "hostname" [:string] :optional
              "hostname of client being bootstrapped")
    (property "file" [:string] :optional
              "fully qualified path of file in zone which will be used to
               bootstrap")
    (property "debug" [:boolean] :optional
              "run the bootstrapping gurp in debug mode"))

  (notes
    "On a bootstrap run, `gurp-user-defs` contains `:is-bootstrap true`, so you
     can change behaviour on an initial run."))

(make-helper-fn "bootstrap" :no-name true)
