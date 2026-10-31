(use judge)
(import ../../../src/doers/file)

(deftest file/checksum
  (test
    (file/checksum
      :algorithm "sha256"
      :literal "6b44cfcc5eb5a878424ac11da95a598a7f7952604bbad2d901c394456d527d02")
    [:checksum
     @{:algorithm "sha256"
       :from-helper true
       :literal "6b44cfcc5eb5a878424ac11da95a598a7f7952604bbad2d901c394456d527d02"
       :unique true}]))

# (deftest file/checksum-error
#   (test-error
#     (file/checksum
#       :algorithm "blake3")
#     "In file/checksum NO-NAME: need exactly one of :literal, :from-url, :with-suffix")

#   (test-error
#     (file/checksum
#       :algorithm "sha256"
#       :literal "6b44cfcc5eb5a878424ac11da95a598a7f7952604bbad2d901c394456d527d02"
#       :safety "not bad")
#     "In file/checksum NO-NAME: unexpected property :safety. Valid properties are :algorithm, :from-url, :label, :literal, :with-suffix")

#   (test-error
#     (file/checksum
#       :algorithm "des"
#       :literal "4")
#     "In file/checksum NO-NAME: algorithm must be one of \"sha256\", \"sha512\", \"blake3\" [Got 'des']")

#   (test-error
#     (file/checksum
#       :algorithm "sha256"
#       :literal "6b44cfcc5eb5a878424ac11da95a598a7f7952604bbad2d901c394456d527d02"
#       :from-url "http://example.com/sum.sha251")
#     "In file/checksum NO-NAME: need exactly one of :literal, :from-url, :with-suffix"))
