(use judge)
(use ../src/commands)
(import ../src/formatting)

(deftest lay-out-help
  (test
    (formatting/lay-out-help
      "test-doer"
      "this is a sample test for the description wrapper test that ought to be
      nicely wrapped to 80 columns with a lovely clean indent and no weird
      spacing in the middle of a line."
      20
      80)
    @["         test-doer  this is a sample test for the description wrapper test that"
      "                    ought to be nicely wrapped to 80 columns with a lovely clean"
      "                    indent and no weird spacing in the middle of a line."]))
