(use ./lib)
(use ../dsl)
(import ../doer-components :as component)
(import ../collector)

(defdoer :system-cert
  "Add and remove system TLS certificates"
  :name-is "File certificate will have in /etc/ssl/certs"
  :exactly-one-of [:content :from :from-url]

  :optional-props-ensure
  {:from {:types [:string]
          :help "Copy cert content from this file. If relative, looks in
                 `../files`"}
   :content {:types [:string]
             :help "Use this literal string as the cert"}
   :url-is-server {:types [:boolean]
                   :help "Used internally to identify Gurp server URLs"}
   :from-url {:types [:string]
              :help "Fetch cert from the given URL"}
   :checksum (component/checksum)}

  :notes
  ["Does not generate certs: just copies them to the system directory and
    re-hashes it."
   "If a `:from` path is relative, Gurp will fully qualify it using the same
    rules as the `file` doer."])

(defn ensure
  [name & spec]
  (pinpoint-error
    :ensure
    (if-not (has-exactly-one-of? [:content :from :from-url] spec)
      (error "Provide exactly one of :content, :from, :from-url")))

  (let [spec-struct (make-spec-struct ;spec)
        spec-table (expand-from-value (struct/to-table spec-struct))
        safe-specs (pinpoint-error
                     :ensure
                     (checked-spec spec-table
                                   mandatory-props-ensure
                                   optional-props-ensure))]

    (collector/push :ensure doer (spec->resource doer name safe-specs))))

(make-remove-fn "system-cert")
