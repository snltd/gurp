(use judge)
(use ../../src/commands)
(use ../../src/doer-docs/describe-docs)

(deftest lay-out-help
  (test
    (lay-out-help
      "test-doer"
      "this is a sample test for the description wrapper test that ought to be
      nicely wrapped to 80 columns with a lovely clean indent and no weird
      spacing in the middle of a line."
      20
      80)
    @["         test-doer  this is a sample test for the description wrapper test that"
      "                    ought to be nicely wrapped to 80 columns with a lovely"
      "                    clean indent and no weird spacing in the middle of a line."]))

(deftest code-example
  (test (code-example string :file :ensure) "(file/ensure \"/example/file/from-content\"\n             :owner \"sys\"\n             :mode \"0600\"\n             :content \"words\\n and\\nstuff\\n\")\n(file/ensure \"/example/file/from-local-file\"\n             :group \"daemon\"\n             :mode \"4755\"\n             :from \"file-dir/example\")\n(file/ensure \"/example/file/from-url\"\n             :label \"remote-file\"\n             :with-checksum \"561a47aa1d1bfc3a95ce45345639f9ce2d9ad332b05cfe5da74ad77f2842ee16\"\n             :from-url \"https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt\")\n")
  (test (code-example string :file :remove) "(file/remove \"/path/to/file\")\n")
)
