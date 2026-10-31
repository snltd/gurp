(use ./definers)
(import ./file/checksum :prefix "" :export true)
(import ../collector)

(defdoer :system-cert
  "Add and remove system TLS certificates"
  :name-is "File certificate will have in /etc/ssl/certs"
  (ensure
    :exactly-one-of [:content :from :from-url]
    (property "from" [:string] :optional
              "Copy cert content from this file. If relative, looks in `../files`")
    (property "content" [:string] :optional
              "Use this literal string as the cert")
    (property "from-url" [:string] :optional "Fetch cert from the given URL")
    (property "url-is-server" [:boolean] :optional
              "Used internally to identify Gurp server URLs")
    (property "checksum" [:struct] :optional "See file/checksum"))

  (notes
    "Does not generate certs: just copies them to the system directory and
     re-hashes it."
    "If a `:from` path is relative, Gurp will fully qualify it using the same
     rules as the `file` doer."))

(make-fn :ensure :system-cert)
(make-fn :remove :system-cert)
