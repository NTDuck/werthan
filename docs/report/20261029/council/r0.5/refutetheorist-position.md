# RefuteTheorist — Round 0.5: Protocol Self-Attack (Target: `01-meta-strategy.md`)

**Persona.** RefuteTheorist (formal-claims soundness: search theory under non-stationary
noisy rewards, reward gating, state-space complexity, liveness). Per `01` §7
self-review requirement, the same adversarial standard is applied to the protocol
document itself. Verified this session (web + E1 file reads); all anchors below.

## Steelman

The protocol (`docs/report/20261029/01-meta-strategy.md`) claims, in its strongest
form:

- **PT-1 (Mechanical convergence).** "A round is **fatal-free** iff the round's ledger
  contains zero findings in FATAL severity with disposition other than
  *rejected-with-counter-evidence*" (§5.5, line 356-358), with convergence = two
  consecutive fatal-free rounds, ≥3 attack rounds, all F-type falsifiers attempted,
  tiers stamped (line 360-363). Strongest form: this is a mechanical, third-party-
  executable stop criterion immune to social convergence pressure.
- **PT-2 (Tiered evidence with named E2 anchors).** "E2 | Peer-reviewed / published,
  verified against primary source this engagement | Harel 1987; UCT and its regret
  pathologies; StateFlow; MAST failure-mode frequencies" (§4 table, line 234). The
  protocol names its own load-bearing E2 anchors, so critics can verify them.
- **PT-3 (No weighted scalarization).** "Axes are *not* collapsed into a weighted sum
  (any weighting is itself a contestable judgment; §3 forbids it)" (line 164-165);
  dominance is vector-dominance with matched evidence tiers (§3, line 190).
- **PT-4 (Falsifier-typed RQs).** Each RQ typed F/P/S with named falsifiers (§1);
  "An F-type RQ whose falsifier was never attempted is not closed" (line 135).
- **PT-5 (Self-referential risk register).** §7 K1–K8 tripwires apply the adversarial
  standard to the critique process itself (line 406+).
- **PT-6 (Oracle-relativity).** "All success-rate measurements in this repo are
  oracle-relative... raw rates are reported alongside hardened rates, never alone"
  (line 258-261).

These are genuinely strong commitments; my objections below attack gaps in the
machinery, not the intent.
