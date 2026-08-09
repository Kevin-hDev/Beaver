# Disabled image-size adapter

`pptxgenjs` 4.0.1 declares `image-size`, but its published runtime does not
import or call it. Beaver's presentation tool also accepts text slides only.

The latest published `image-size` release is affected by
GHSA-w3rx-r6r6-pgpr and GHSA-5p2g-fcmc-qvqq, with no patched release. This
local package replaces that unused parser and fails closed if code ever tries
to inspect an image.

Remove this adapter only after the production presentation tests pass with a
patched upstream dependency, or after `pptxgenjs` removes the dependency.
