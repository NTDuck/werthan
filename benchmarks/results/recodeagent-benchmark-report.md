# ReCodeAgent Benchmark Report

Run date: 2026-10-18 · Runner: `werðan/daemon --benchmark-all` · Dataset: `benchmarks/assets/ReCodeAgent/data/tool_projects`
Inference: `qwen3.8-27b` via ashalliants-df2 (RTX 3090, real HTTP completion, 118/118 real)

## Overall

| Metric | Value |
|---|---|
| Total projects benchmarked | 118 |
| Overall compilation success rate | 90.68% |
| Overall mean test pass rate | 8.95% |
| Mean wall time per project | 1713.51 ms |
| Total wall time | 202.26 s |

## Per-Category Breakdown

| Category | Projects | Compilation rate | Mean test pass rate | Mean wall time (ms) | Passed |
|---|---|---|---|---|---|
| crust | 100 | 99.00% | 3.56% | 2009.35 | 2 |
| oxidizer | 6 | 0.00% | 0.00% | 48.67 | 0 |
| alphatrans | 4 | 0.00% | 0.00% | 111.50 | 0 |
| skel | 8 | 100.00% | 87.50% | 65.12 | 7 |

## Per-Project Results

| Project | Category | Wall time (ms) | Compilation | Test pass rate | Receipts | Inference | Status |
|---|---|---|---|---|---|---|---|
| commons-cli | alphatrans | 100 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| 2dpartint | crust | 192 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| checkdigit | oxidizer | 42 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| bst | skel | 53 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| commons-csv | alphatrans | 111 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| 42-kocaeli-printf | crust | 171 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| go-edlib | oxidizer | 49 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| colorsys | skel | 63 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| commons-fileupload | alphatrans | 121 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| aes128-simd | crust | 194 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| gohistogram | oxidizer | 48 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| heapq | skel | 55 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| commons-validator | alphatrans | 114 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| amp | crust | 218 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| gonameparts | oxidizer | 42 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| html | skel | 57 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| approxidate | crust | 2311 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| stats | oxidizer | 66 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| mathgen | skel | 121 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| avalanche | crust | 232 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| textrank | oxidizer | 45 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| rbt | skel | 61 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| bhshell | crust | 277 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| strsim | skel | 57 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| bigint | crust | 295 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| toml | skel | 54 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| bitset | crust | 272 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| blt | crust | 3027 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| bostree | crust | 2523 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| btree-map | crust | 278 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| c-aces | crust | 4413 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| c-blind-rsa-signatures | crust | 6451 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| c-string | crust | 317 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| carrays | crust | 2769 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| cfsm | crust | 247 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| chtrie | crust | 287 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| circularbuffer | crust | 237 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| cissy | crust | 281 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| cjson | crust | 300 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| clhash | crust | 208 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| clog | crust | 2244 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| coroutine | crust | 3771 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| cset | crust | 286 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| csyncmers | crust | 2475 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| dict | crust | 392 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| emlang | crust | 271 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| expr | crust | 626 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| fasthamming | crust | 264 | ✅ | 16.7% | 4/6 | simulated | FAILED |
| fft | crust | 238 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| file2str | crust | 231 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| fleur | crust | 2470 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| fs_c | crust | 281 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| fslib | crust | 501 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| genetic-neural-network-for-simple-control | crust | 3041 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| geofence | crust | 298 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| gfc | crust | 272 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| gorilla-paper-encode | crust | 240 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| graph-recogniser | crust | 297 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| hamta | crust | 259 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| holdem-odds | crust | 3032 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| hydra | crust | 247 | ✅ | 50.0% | 6/6 | simulated | FAILED |
| impcheck | crust | 120465 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| inversion_list | crust | 708 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| jccc | crust | 368 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| kairocompiler | crust | 394 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| kd3 | crust | 323 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| lambda-calculus-eval | crust | 379 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| leftpad | crust | 202 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| lib2bit | crust | 337 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libbase122 | crust | 378 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libbeaufort | crust | 233 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libfor | crust | 415 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| libm17 | crust | 2586 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libpgn | crust | 464 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libpsbt | crust | 2240 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libqueue | crust | 241 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libtinyfseq | crust | 268 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| libutf | crust | 206 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libvcd | crust | 290 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| libwecan | crust | 296 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| linear-algebra-c | crust | 323 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| ljmm | crust | 289 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| ltre | crust | 332 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| math-library-in-c | crust | 272 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| matrix_multiplication | crust | 3422 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| mdb | crust | 2583 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| megalania | crust | 2554 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| merkle-tree-c | crust | 362 | ✅ | 16.7% | 4/6 | simulated | FAILED |
| morton | crust | 223 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| murmurhash_c | crust | 206 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| mvptree | crust | 3695 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| nandc | crust | 255 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| phills_dht | crust | 244 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| quadtree | crust | 246 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| razz_simulation | crust | 326 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| rbtree-lab | crust | 312 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| recordmanager | crust | 354 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| rect_pack_h | crust | 276 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| remimu | crust | 268 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| rhbloom | crust | 300 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| roaring-bitmap | crust | 242 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| rubiksolver | crust | 329 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| satc | crust | 289 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| simple-config | crust | 295 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| simple-sparsehash | crust | 293 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| simple_lang | crust | 276 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| simplexml | crust | 298 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| skp | crust | 287 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| slothlang | crust | 243 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| ted | crust | 1175 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| tisp | crust | 370 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| totp | crust | 238 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| ulidgen | crust | 257 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| utf8 | crust | 300 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| vaultsync | crust | 254 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| vec | crust | 309 | ✅ | 72.7% | 6/6 | simulated | FAILED |
| worsp | crust | 275 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| xopt | crust | 334 | ✅ | 0.0% | 4/6 | simulated | FAILED |
