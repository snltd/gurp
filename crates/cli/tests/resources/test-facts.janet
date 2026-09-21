#
# Run very basic tests against all our facts. This should only run on illumos,
# otherwise almost everything will fail. I want to test hitting a real system,
# which means returned values are unpredictable, hence the vagueness of the
# tests.
#
# Run as part of the facts acceptance test.
# 
(defn- test-simple-val [val-to-test val-type]
  (assert (= (type val-to-test) val-type)
          (string/format "val-to-test is wrong type: expected %s got %s"
                         val-type
                         (type val-to-test)))
  (assert (< 2 (length val-to-test)
             "val-to-test is too short")))

(defn- test-struct-val
  [val-to-test expected-keys]
  (assert (= (type val-to-test) :struct)
          "val-to-test is not a struct")
  (loop [[k v] :pairs val-to-test]
    (assert (deep= (sorted (keys v)) (sorted expected-keys))
            (string/format "key mismatch: expected %s\n      got %s"
                           (sorted expected-keys)
                           (sorted (keys v))))))

(role test-hostname
      (test-simple-val (fact :hostname) :string))

(role test-zonename
      (test-simple-val (fact :zonename) :string))

(role test-ip-addresses
      (test-struct-val (fact :ip-addresses) [:type :state :addr]))

(role test-ip-interfaces
      (test-struct-val (fact :ip-interfaces) [:class :state :current :persistent]))

(role test-links
      (test-struct-val (fact :links) [:class :mtu :state :bridge :over]))

(role test-physical-links
      # This doesn't give you anything, generally, in a NGZ, so we'll just check
      # it doesn't error
      (assert (= (type (fact :physical-links)) :struct)))

(role test-uname
      (let [val-to-test (fact :uname)]
        (assert (deep= (sorted (keys val-to-test))
                       (sorted [:serial :machine :numcpu :node :bustype :oem
                                :users :release :origin :kernelid :system])))))

(role test-zfs-filesystems
      (test-struct-val (fact :zfs-filesystems)
                       [:used :avail :refer :mountpoint]))

(role test-zone-brand
      (test-simple-val (fact :zone-brand) :keyword))

(role test-zones
      (test-struct-val (fact :zones)
                       [:id :status :path :brand :ip]))

(role test-zpools
      (test-struct-val (fact :zpools)
                       [:size :alloc :free :ckpoint :expandsz :frag :cap :dedup
                        :health :altroot]))

(host "test"
      (test-hostname)
      (test-ip-addresses)
      (test-ip-interfaces)
      (test-links)
      (test-physical-links)
      (test-uname)
      (test-zfs-filesystems)
      (test-zone-brand)
      (test-zones)
      (test-zpools)
      (test-zonename))
