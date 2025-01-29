# gurp

Gurp is an almost certainly doomed attempt to write a configuration management
tool in Lisp.

## Design

### Important

* Configuration is written in Janet.
* Coverage of everything I need in OmniOS.
* Speed: runs must complete quickly.
* Clear reporting at the end of a run.

### Maybe
* You (or some program really) compiles a single binary containing Janet,
gurp's "doers", and the machine configuration. You can then plonk that binary
on your host and run it.

### Not Important
* Any kind of back-end or server.
* Clever ordering of resources - I might take a crude multiple-run approach.
* Coverage of anything I don't need in OmniOS.
* Coverage of anything that isn't OmniOS.
