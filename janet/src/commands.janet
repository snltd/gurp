#
# Interfaces called by the Gurp binary.
# 
(use ./doer-docs/describe-docs)
(use ./doer-docs/formatting)
(use ./doer-docs/lib)
(use ./doers)

(defn help-for
  "Called by the Rust 'describe' command"
  [object]
  (try
    (print
      (if (string/find "/" object)
        (help-for-sub-resource (splice (string/split "/" object 0 2)))
        (help-for-doer object)))
    ([_e]
      (eprint "No help for '" object "'"))))

(defn list-doers
  "Returns a multiline string, pairing doers with their descriptions. Used by
  Gurp's 'doers' command"
  []
  (def descriptions [["apk" apk/description]
                     ["bridge" bridge/description]
                     ["cron" cron/description]
                     ["directory" directory/description]
                     ["etherstub" etherstub/description]
                     ["file-line" file-line/description]
                     ["file" file/description]
                     ["gem" gem/description]
                     ["group" group/description]
                     ["ip-address" ip-address/description]
                     ["ip-interface" ip-interface/description]
                     ["ip-properties" ip-properties/description]
                     ["ipnat" ipnat/description]
                     ["misc" misc/description]
                     ["network-flow" network-flow/description]
                     ["pkg" pkg/description]
                     ["pkgin" pkgin/description]
                     ["publisher" publisher/description]
                     ["route" route/description]
                     ["smf" smf/description]
                     ["svc" svc/description]
                     ["svcprop" svcprop/description]
                     ["symlink" symlink/description]
                     ["user" user/description]
                     ["vlan" vlan/description]
                     ["vnic" vnic/description]
                     ["zfs" zfs/description]
                     ["zone" zone/description]])

  (string/join
    (flatten
      (map |(lay-out-help ;$ 25 (term-width)) descriptions))
    "\n"))
