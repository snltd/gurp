(use ./lib)
(import ../collector)

(defdoer :ipnat
  "Set or remove NAT rules."
  :name-is "Any convenient name: not used internally"
  :exactly-one-of [:content :from]

  :mandatory-props-ensure
  {:priority {:types [:number]
              :help "NAT rule resources are ordered by priority, lowest number
                     first"}}

  :optional-props-ensure
  {:from {:types [:string]
          :help "Apply rules in the given file. If relative, looks in
                 `../files`"}
   :content {:types [:string]
             :help "Apply these rules"}}

  :notes
  ["Gurp assembles bundles of NAT rules, given as literal strings (`:content`)
    or in files (`:from`). Ordering comes from the `:priority` value, lowest
    first. When all rules have been assembled, the list is verified and compared
    against the currently loaded NAT rules. If different, the new rules are
    applied and written to `/etc/ipf/ipnat.conf`."
   "Every run asserts the live and persistent state of the NAT table."
   "No ipnat flags (-R, -r etc) are supported."
   "It's too tricky to support local-zone-from-global-zone NAT rules, so we
    don't."
   "The doer automatically enables the ipfilter service."
   "ipnat/remove removes ALL NAT rules"])

(make-ensure-fn "ipnat")
(make-remove-fn "ipnat")
