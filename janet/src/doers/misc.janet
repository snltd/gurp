(use ./definers)
(use ../dsl)
(import ../collector)

(defdoer :misc
  "A collection of things too small to deserve their own doer."
  (ensure
    (property "enable-smb" [:string] :optional
              "Enable SMB sharing for this username")
    (property "nfs-domain" [:string] :optional
              "NFS domain name")
    (property "scheduler" [:string] :optional
              "The scheduler class to set via dispamdin"))

  (notes
    `The misc doer is a placeholder for what Gurp considers "OS primitives"
     but which are not big or complex enough to warrant their own doer`))

(defn ensure
  "Sets miscellaneous system properties"
  [& spec]
  (let [name (labelise spec)]
    (pinpoint-error
      :ensure
      (collector/push :ensure doer (make-ensure-resource)))))
