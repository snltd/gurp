# Compile Test Fixtures

These things are probably more trouble than they're worth, because they're
brittle. But they have caught at least one real bug, so I'm keeping them around
for now.

If the compile acceptance tests fail but in a way you expect from a change, you
can regenerate the output fixtures:

```fish
for f in inputs/*janet
  ../../../../../target/debug/gurp compile \
    --format json \
    --output-file outputs/(basename $f janet)json \
    $f
  end
```
