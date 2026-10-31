(import ../../collector)
(use ../definers)

(def algorithms ["sha256" "sha512" "blake3"])

(defhelper :file :checksum
  "Define a checksum to be used when fetching a remote file"
  :exactly-one-of [:literal :from-url :with-suffix]
  :unique true
  (helper
    (property "algorithm" [:string] :mandatory
              "Algorithm used to calculate checksum"
              :values algorithms)
    (property "literal" [:string] :optional "Literal checksum")
    (property "from-url" [:string] :optional "Fetch checksum from this URL")
    (property "with-suffix" [:string] :optional
              "When fetching a file from a URL, append this suffix to fetch the
             checksum")))

(make-helper-fn :checksum)
