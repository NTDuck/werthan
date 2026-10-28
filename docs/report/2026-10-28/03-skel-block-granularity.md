# 03 — skel Block Granularity: The Translation Unit the Methodology Doesn't Model

The eight skel projects (`tool_projects/skel/{bst,colorsys,heapq,html,mathgen,rbt,strsim,toml}`) are shipped as **python source + a JS translation sliced into numbered blocks** (`/// --- BLOCK BEGIN n` … `/// --- BLOCK END n`). The block is ReCodeAgent's de-facto translation unit: the translator fills one block at a time. Reading whole pairs shows what block-level translation structurally cannot see.

## Per-project shape

| project | source.py LoC | translated.js LoC | JS block markers | scaffolding (SKEL HEAD) |
|---|---|---|---|---|
| bst | 808 | 364 | 19 | 95/364 = 26.1% |
| colorsys | 355 | 285 | 8 | 33.3% |
| heapq | 497 | 494 | 22 | 19.2% |
| html | 1,479 | 1,173 | 44 | 8.1% |
| mathgen | 1,636 | 1,365 | 81 | 7.0% |
| rbt | 906 | 714 | 27 | 13.3% |
| strsim | 1,591 | 1,400 | 64 | 6.8% |
| toml | 2,060 | 2,051 | 72 | 4.6% |

Every file carries an identical 95-line `SKEL HEAD` shim (lines 2–96): `user_check_type(obj, _type)` + `SkelClass(name)` (`translated.js:90-95`: `let _class_var = {}; _class_var._class_name = name; return _class_var;`). All 8 head shims contain the same magic sentinel with **no counterpart in any Python source**: `return Number.isSafeInteger(obj) && obj !== 1e6;` (bst `translated.js:69`) — exactly one integer value classified as "not an int".

**Systematic omissions from translated.js**: all test blocks and global block 0 (module-level definitions) exist only in `source.py`. No `module.exports`, no `test()` call. mathgen's artifact additionally cannot load (`tracer_skip.js:2` `const { exp } = require('numjs');` — package absent). The verification loop is invisible to the artifact.

## Five coupling classes block translation cannot preserve

### 1. Late-bound `class_var` and assembly sections
Every method block closes over a `class_var` declared *after* the blocks, plus a manual attachment list outside any block:
- bst: blocks 1–18 reference `class_var`, declared at `translated.js:324` (`var class_var = SkelClass('BinarySearchTree');`), 17 attachment lines follow; the `Node` factory binds at `:109` and immediately calls `__init__(param_0, param_1)`.
- rbt: 26 method blocks close over `class_var` declared at `translated.js:673`, 26 attachments follow.
- html: inheritance encoded as `class_var._class_name = 'HTMLParser;' + class_var._class_name;` (`translated.js:1145-1146`), consumed by the head shim's `_class_name.split(";")` loop — subclass dispatch depends on assembly code, not any block.

A block-local translator sees free variables; it cannot know which object the block mutates, that state lives in per-instance closures, or that inheritance is a string-chain protocol.

### 2. Dropped global block 0 and module-level mutable state
- html: `source.py:1371-1470` defines `name2codepoint`, `entitydefs`, `CDATA_CONTENT_ELEMENTS`, `listener_event_list = []`, 20+ prebound regex methods — none in translated.js; line 1 imports 21 constants from `./tracer_skip.js` (out-of-band). `CDATA_CONTENT_ELEMENTS` is referenced by blocks (`:624`) but never defined in the js.
- colorsys: blocks 4/5 use `ONE_THIRD/ONE_SIXTH/TWO_THIRD` (`translated.js:185-204`) defined only in py global block 0 (`source.py:337-341`) — bare load + call throws.
- html's `listener_event_list` is a cross-call accumulating observer log; the blocks appending to it are dropped.
- strsim: block 14 inlines `var no_space_str = string.replace(/\s+/g, " ");` (`translated.js:455`) where py used module-level `_SPACE_PATTERN` — the block silently depends on a definition it doesn't contain.

### 3. Cross-block call/type contracts
- Call graphs span blocks: bst `put` (5) → `_put` (6) → `Node` (1); `remove` (9) → `_reassign_nodes` (10) ← `_get_lowest_node` (11). toml's `loads`→`handle_*` closures share ~15 `nonlocal` variables.
- heapq: min-heap side patched via `require('./tracer_skip.js')` inside blocks 9/10 (`translated.js:189, :208`) — the max-heap sibling blocks 11/12 (`:230-269`) keep raw `<` on arrays; py's lexicographic tuple comparison (`source.py:181-199`) silently loses tie-breaking for `nsmallest`/`nlargest` (15/16).
- rbt: `NotImplemented` is a bare identifier in JS (`translated.js:662`) — undefined at runtime; block 2's duck-typing rewrite `(!parent.left.__eq__ && parent.left === class_var)` dereferences `.__eq__` on possibly-null nodes py guarded with `hasattr` (`source.py:88`).
- strsim's Damerau `str(0)`/`int()` sentinel round-trip (`source.py:281-300`) became `'0'`/`parseInt(...)` (`translated.js:395, :415`) — unseen chars yield `parseInt(undefined) → NaN` → silent NaN distances instead of py's KeyError.

### 4. Per-language semantic divergences, decided block-locally
- bst throws `new Exception(msg)` at `:158/:177/:262` (undefined constructor → ReferenceError) but `new Error(...)` at `:276` for the symmetric `get_min_label` — py source uses identical `raise Exception(...)` in both. The catching block (12) swallows any exception, so the divergence is invisible to the other block.
- colorsys: py `h = (h/6.0) % 1.0` (non-negative modulo) vs js `(h / 6.0) % 1.0` + hand-repair `if (h < 0) { h += 1.0; }` (`translated.js:171-173`) — a whole-function fix impossible from block-local view; sibling `parseInt(h * 6.0)` at `:248` vs py truncating `int()` — `parseInt(-0.9) → NaN`.
- mathgen: 145 randomness calls route to `user_randint`/`user_choice_func1/2`/`user_sample_func1/2`/`user_uniform` implemented in the side file (`tracer_skip.js`, seeded `var seed = 1`); py test block 85 interleaves `user_reset_seed()` — the reproducibility invariant lives in dropped blocks + side file.
- mathgen asserts Python-repr strings (`assert_equal(b, '0.49 radians')`, `source.py:1400`); JS number→string formatting differs from py f-strings for many doubles.

### 5. The verification loop is external to the artifact
All test/assert blocks exist only in `source.py`; patches counted as `user_fix` (heapq `tracer_skip.js:1-2`) and dataset-specific hacks (toml's `"1979"` literal matching `example.toml`, `translated.js:1032`) sit outside every block. Per-block translation quality is not measured against the executable behavior the dataset claims to test — html's node probe timed out (>15 s) at 18:58 on 2026-10-25 while the campaign was running.

## Methodology consequence

METHODOLOGY.md's DAG vertex is "a single file/module or a collapsed SCC" (§2, Def 1). The skel corpus demonstrates the real unit of failure is **intra-file: a block whose correctness depends on other blocks' initialization, throwing behavior, comparison operators, and module state — plus assembly code no block contains**. A per-vertex hydration ($\mathcal{G}_{\text{slice}}$ 1-hop) that models vertices as files cannot surface these couplings; the $\mathcal{A}_{\text{patch}}$ fault-localization loop receives symptoms ("test X fails") whose causes live in a different block's initialization order or a side file. Whether the <...> methodology needs sub-file translation units (or an explicit assembly-contract artifact per file) is a design decision this evidence forces.
