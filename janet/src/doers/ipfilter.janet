(use ./lib)
(import ../collector)

(defdoer :ipfilter
  "Set or remove ipfilter rules."
  :name-is "Any convenient name: not used internally"

  :mandatory-props-ensure
  {:priority {:types [:number]
              :help "rule resources are ordered by priority, lowest number first"}
   :always-reload {:types [:boolean]
                   :help "if any ipfilter/ensure resource sets this to true, then the
                    firewall rules will be reloaded every time Gurp runs,
                    regardless of whether the aggregated ipf.conf file has changed"}}

  :optional-props-ensure
  {:from {:types [:string]
          :help "Apply rules in the given file. If relative, looks in ../files"}
   :content {:types [:string]
             :help "Apply these rules. Must have :content xor :from"}}

  :defaults-ensure
  {:always-reload false}

  :notes
  ["We build a single big set of filter rules from multiple sources, check its
    validity, and ensure its contents align with those of `/etc/ipf/ipf.conf`.
    If the file has changed, or if any resource used to build the content has
    `:always-reloaded true`, the contents of the file become the current
    firewall configuration."
   "The doer automatically enables the ipfilter service."
   "We do not (currently) support any additional `ipf` options."
   "Per-zone rules are not supported."
   "Using :always-reload means Gurp will always show a change to be made"
   "ipfilter/remove removes ALL filter rules"])

(defn ensure
  [name & spec]
  (if (has-exactly-one-of? [:content :from] spec)
    (collector/push :ensure doer (make-ensure-resource))
    (pinpoint-error
      :ensure
      (error "need exactly one of :content or :from"))))

(defremove "ipfilter")
