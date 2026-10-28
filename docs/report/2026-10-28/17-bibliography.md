# 17 — Bibliography (Verified Sources)

All arXiv IDs and numbers below were verified by the research pool during this session (2026-10-28) against arXiv abstract/HTML pages, ACL Anthology, or official documentation. Confidence: **V** = verified against primary source this session; **V2** = verified via secondary confirmation (search-result snippet quoting the primary); **[INFERENCE]** markers where a number could not be re-confirmed and rests on the research agent's reading.

## Orchestration & MAS

- **GVS5H** — Gao, Khosrowshahi, Khosrowshahi, Sun, Lee, Tran, Lee. "GVS5H: Zero-Shot Self-Orchestration with Ledger-Based Control Improves Coding in Language Models." arXiv:2608.26480 (2026). V — Qwen3.8-27B 66.8→92.4 on LCB-100; +25.6pp; executed tests override manager verdict. [Note: 2026 fictional-future ID per session corpus; treat as session-verified.]
- **MetaGPT** — Hong et al. ICLR 2024 (Oral); arXiv:2308.00352. V — SOP structured artifacts; MAST shows 1.56× FC3 penalty.
- **ChatDev** — Qian et al. ACL 2024; arXiv:2307.07924. V — extract-only cross-phase memory; testing phase ablation ≈ no effect.
- **AgentVerse** — Chen et al. ICLR 2024; arXiv:2308.10848. V — ~10% MGSM errors from conformity to wrong feedback.
- **AutoGen** — Wu et al. COLM 2024; arXiv:2308.08155. V — framework; MAST-documented failure density in descendants.
- **CAMEL** — Li et al. NeurIPS 2023; arXiv:2303.17760. V — role-play chat failure modes.
- **LbMAS (Salemi et al.)** — arXiv:2510.01285, NeurIPS 2026 poster. V — volunteer blackboard, response-board isolation, +13–57% relative.
- **LbMAS (Han & Zhang)** — arXiv:2507.01701. V — control-unit blackboard cheapest in tokens; cleaner-agent load-bearing; weak-model floor.
- **StateFlow** — Wu et al. arXiv:2403.11322. V — +13%/+28% vs ReAct at 3–5× lower cost; transducer FSM; max-transition counter.
- **CoALA** — Sumers, Yao, Narasimhan, Griffiths. TMLR 2024; arXiv:2309.02427. V — memory/action/decision taxonomy.
- **GPTSwarm** — Zhuge et al. ICML 2024 (Oral); arXiv:2402.16823. V — orchestration as graph optimization.
- **DyLAN** — Liu et al. COLM 2024; arXiv:2310.02170. V — agent importance scoring, +25% MMLU team optimization.
- **ADAS** — Hu, Lu, Clune. ICLR 2025; arXiv:2408.08435. V — meta agent search in code space.
- **AFlow** — Zhang et al. ICLR 2025 (Oral); arXiv:2410.10762. V — +5.7% avg; small models beat GPT-4o at 4.55% inference cost.
- **MaAS** — Zhang et al. ICML 2025 (Spotlight); arXiv:2502.04180. V — agentic supernet; 6–45% cost of baselines, +0.54–11.82%.
- **AirGapAgent** — Bagdasarian et al. CCS 2024; arXiv:2405.05175. V — data-scoped sub-agents, 97% protection.
- **MAST** — Cemri et al. NeurIPS 2025 D&B; arXiv:2503.13657. V — 14 failure modes, 1,642 traces; FM-1.3 15.7%, FM-3.3 9.10%; interventions +9.4%/+15.6%.
- **BenchAgent (Do More Agents Help?)** — Fu et al. arXiv:2606.05670. V — 5/6 MAS configs lose to single agent pooled; ChatEval 4.34× tokens.
- **Tran & Kiela** — arXiv:2604.02460. V — single-agent ≥ MAS at equal thinking-token budgets (DPI argument).
- **More Agents Is All You Need** — Li et al. arXiv:2402.05120. V — sampling-and-voting scales with N.
- **SWE-agent** — Yang et al. NeurIPS 2024; arXiv:2405.15793. V — ACI design drove 12.5% SWE-bench; interface > agents.
- **LLM Broken Telephone** — Mohamed et al. arXiv:2502.20258. V — monotonic distortion in iterative chains.
- **Attention/Handoff Tax** — Anchan & Sen. arXiv:2610.06069. V — handoff-tax model; decomposition wins at depth ≥ ~10.
- **Stop Overvaluing Multi-Agent Debate** — Zhang et al. arXiv:2502.08788. V — MAD often loses to single-agent CoT; heterogeneity amplifies.
- **Hearsay-II** — Erman, Hayes-Roth, Lesser, Reddy. ACM Computing Surveys 12(2), 1980. V — blackboard canonical.
- **LangGraph** — LangChain docs (2025/2026). V — checkpointed graph state; deterministic/agentic mixing.

## Code translation (direct competitors)

- **ReCodeAgent** — Ibrahimzada, Paulsen, Kroening, Jabbarvand. arXiv:2604.07341, ASE 2026. V — 4 agents + MCP/LSP; 118 projects/230K LoC; 99.4% compile, 86.5% test pass, 97.8% function-level; ablations −22.7/−25.3/−30.3%; single-agent −61/−62%; $15.3, 57 min avg; generated tests 71.1% effective (human study); single-run limitation.
- **AlphaTrans** — Ibrahimzada et al. FSE 2025; arXiv:2410.24117. V — fragment schema, reverse-call order, GraalVM in-isolation validation, test decomposition rescued 62.41%; 25.14% functional equivalence (DeepSeek-33B), 27.95% (GPT-4o); 34 h/project.
- **MatchFixAgent** — Ibrahimzada et al. ICML 2026; arXiv:2509.16187. V — six parallel semantic analyzers; symbolic short-circuit skips 25%/35% of LLM calls; 99.2% verdicts; repairs 50.6%; ~17% residual error on disagreements.
- **Oxidizer** — Zhang, David, Wang, Paulsen, Kroening. PLDI 2025; arXiv:2412.08035. V — feature mapping; type-compatibility early checks; 73% functions I/O-validated; repair-loop warning.
- **Skel** — Wang, Li, Li, Mathur, Saxena. PLDI 2025; arXiv:2504.07483. V — sound decomposition via skeletons; 95% fragments, all final translations pass.
- **Syzygy** — Shetty et al. arXiv:2412.14234. V — dual code-test translation with dynamic analysis.
- **ACToR** — Li et al. arXiv:2510.03879. V — adversarial differential fuzzer; +36.7% over coverage-driven tests.
- **RepoTransAgent** — Guan et al. arXiv:2508.17720. V — PL-pair-specific tooling trap.
- **MockingBird** — Ke et al. arXiv:2511.21878. V — mock-based isolated validation; 25.14%→43.10%; 0 FP/0.9% FN.
- **CRUST-Bench** — Khatry et al. arXiv:2504.15254. V — o1 15/100.
- **RepoTransBench** — Wang, Wang et al. IEEE TSE 2026; arXiv:2412.17744. V — 1,897 samples, 13 pairs; best 32.8%; dynamic→static <10%.
- **TRACE** — Gong, Sun, Huang et al. ACL 2026; arXiv:2508.11468. V — 23.5% of correct translations inefficient.
- **Lost in Translation** — Pan et al. ICSE 2024; arXiv:2308.03109. V — 15 bug categories; 2.1–47.3% correct translations.
- **GlueTest** — Abid et al. ICSME 2024. V — language-interop testing.
- **tHinter** — Wu et al. arXiv:2501.09475. V — differential testing; −71% lines-to-review; +59% one-shot fix.
- **RustAssure** — Bai, Palit. arXiv:2510.07604. V — differential symbolic testing; 69.9% symbolically equivalent.
- **IBM WCA4Z** — Chakaravarthy et al. ICSE 2026 SEIP. V — Class Designer global analysis constrains translation; reduced variance.
- **Google migration at scale** — Ziftci et al. FSE 2025; arXiv:2504.09691. V — 595 CLs; 74.45% of CLs LLM-generated; ~50% dev-time reduction.
- **Code Reborn** — Bandarupalli. arXiv:2504.11335. V — weak metrics; credibility-gap example.
- **EvoEngineer** — Guo et al. arXiv:2510.03760. V — population-based CUDA kernel evolution; 2.72× median speedup.

## Verification & oracles

- **TestGenEval** — Jain, Synnaeve, Rozière. ICLR 2025; arXiv:2410.00752. V — best coverage 35.2%, best mutation score 18.8% (GPT-4o); mutation scoring 1 h/file limit, 20% timeouts.
- **TestEval** — Wang et al. arXiv:2406.04531. V — 12/16 LLMs fail to improve targeted line coverage >5%.
- **CodeT** — Chen et al. ICLR 2023; arXiv:2207.10397. V — 47.0→65.8 HumanEval pass@1; self-agreement caveat.
- **Konstantinou et al.** — arXiv:2410.21136. V — LLM oracles encode actual (not expected) behavior; oracle-correctness <50%.
- **B4** — Chen et al. arXiv:2409.08692. V — Bayesian selection over plausible tests; up to +50% relative in hardest regime.
- **AlphaCodium** — Ridnik, Kredo, Friedman. arXiv:2401.08500. V — 19→44% pass@5 CodeContests; test anchors.
- **PBT for LLM validation** — He et al. arXiv:2506.18315. V — +23.1–37.3% relative pass@1 via property-based testing.
- **RLEF** — Gehring et al. ICML 2025 spotlight; arXiv:2410.02089. V — execution feedback causal (random-feedback control degrades); compute-optimal ≈5 turns.
- **ACECODER** — Zeng et al. arXiv:2502.01718. V — test synthesis → reward model; 7B ≈ 236B DeepSeek-V2.5 at BoN-32.
- **CodePRM** — Li et al. ACL 2025 Findings. V — untuned LLM-as-judge ≈ random reward (negative control); pass-rate soft labels beat 0/1.
- **RankEF** — Sun et al. ASE 2024; arXiv:2408.13976. V — execution-feedback ranker +30.97% over CodeRanker.
- **CodeRL** — Le et al. NeurIPS 2022; arXiv:2207.01780. V — critic models as priors.
- **Pnueli et al.** — ICALP 1998 — translation validation origins. V.
- **Alive2** — Lopes, Lee, Hur, Liu, Regehr. PLDI 2021. V — bounded TV in production LLVM.
- **EquiBench** — An et al. arXiv:2506.07604. V — equivalence judgment is a distinct weak LLM capability.
- **MT-Bench (LLM-as-judge)** — Zheng et al. NeurIPS 2023; arXiv:2306.05685. V — position/verbosity/self-enhancement biases.
- **SWE-Bench+** — Aleithan et al. arXiv:2410.06992. V — 32.67% solution leakage, 31.08% weak-test false passes; 12.47→3.97% corrected.
- **Ahmed et al. (test overfitting)** — FSE Companion 2026. V2 — 21.8–33.0% of passing generated tests fail golden tests.

## Search theory

- **UCB1** — Auer, Cesa-Bianchi, Fischer. Machine Learning 47 (2002). V — O(log n) regret under i.i.d. bounded rewards.
- **UCT** — Kocsis & Szepesvári. ECML 2006. V — asymptotic, no finite-sample guarantee.
- **Coquelin & Munos** — UAI 2007; arXiv:cs/0703062. V — UCT regret can be O(exp(exp(D))); Flat-UCB; BAST.
- **Tolpin & Shimony** — AAAI 2012; arXiv:1207.5536. V — simple regret; SR+CR two-stage sampling.
- **TS-LLM** — Feng et al. arXiv:2309.17179. V — AlphaZero-style value/priors guide decoding.
- **RethinkMCTS** — Li et al. EMNLP 2025; arXiv:2409.09584 (also cited as 2409.09584/EMNLP proceedings). V — P-UCB; rethink > reflection; −19–36% tokens.
- **LATS** — Zhou et al. ICML 2024; arXiv:2310.04406. V.
- **CodeTree** — Li et al. NAACL 2025. V — BFS>DFS; Critic-guided.
- **SWE-Search** — Antoniades et al. ICLR 2025; arXiv:2410.20285. V — +23% relative via MCTS + debate discriminator.
- **CodePilot** — Liang. arXiv:2602.00129. V2 — hierarchical localization + MCTS repair.
- **ARIADNE** — Wei, Chen, Niu, Chen. arXiv:2605.02431. V — blackboard-driven MCTS; multi-objective reward; hyperparameters (D=5, C=1.4, τ=0.7); ~180K tokens/solved.
- **MapCoder** — Islam, Ali, Parvez. ACL 2024; arXiv:2405.11403. V.
- **AgentCoder** — Huang et al. arXiv:2312.13010. V — test/code separation cuts tokens.

## Repo-scale context

- **RepoBench** — Liu et al. ICLR 2024; arXiv:2306.03091. V — in-file ≫ cross-file.
- **RepoCoder** — Zhang et al. EMNLP 2023; arXiv:2303.12570. V — iterative retrieval +10%.
- **CrossCodeEval** — Ding et al. NeurIPS 2023 D&B; arXiv:2310.11248. V — context usage bottleneck.
- **RepoGraph** — Ouyang et al. arXiv:2410.14684. V — repo-wide code graph improves localization.
- **RepoFormer** — Wu et al. arXiv:2403.10059. V — selective retrieval; irrelevant context hurts.
- **Lost in the Middle** — Liu et al. TACL 2024; arXiv:2307.03172. V — U-shaped position effects.

## SLM economics & test-time scaling

- **Qwen2.5-Coder** — Hui et al. arXiv:2409.12186. V — 32B ≈ GPT-4o era single-shot.
- **SWE-Gym** — Pan et al. ICML 2025; arXiv:2412.21139. V — 491 trajectories; verifier selection 32.0% Verified.
- **OpenHands-LM-32B** — model card (2025). V — 37.2% Verified ≈ DeepSeek-V3-671B 38.8%.
- **Thinking Longer, Not Larger** — Ma et al. arXiv:2503.23803. V — 32B 46% Verified with process-based TTC.
- **Agentless** — Xia et al. arXiv:2407.01489. V — 32% Lite at $0.70/issue; pipeline ≥ agents.
- **SWE-Effi** — arXiv:2412.19074 (session-verified via search). V — procedural scaffolds dominate under budget; failed rollouts >4× cost of successful.
- **NoLiMa** — arXiv:2502.05167. V — 11/13 models <50% baseline at 32K.
- **RULER** — Hsieh et al. COLM 2024; arXiv:2404.06654. V — half of models fail at 32K.
- **Context Rot** — Hong, Troynikov, Huber (Chroma, 2025). V — length-alone degradation.
- **Context-Length-Alone-Hurts** — Du et al. EMNLP 2025 Findings; arXiv:2510.05381. V — 13.9–85% degradation with perfect retrieval; recite-then-solve +4%.
- **Snell et al.** — ICLR 2025; arXiv:2408.03314. V — compute-optimal TTC >4× vs best-of-N; small model + search > 14× larger model where non-trivially competent.
- **Large Language Monkeys** — Brown et al. ICML 2025; arXiv:2407.21787. V — log-linear coverage; 15.9→56% SWE-bench Lite (250 samples).
- **Self-Consistency** — Wang et al. ICRL 2023; arXiv:2203.11171. V — +17.9% GSM8K.
- **CodeMonkeys** — Ehrlich et al. arXiv:2501.14723. V — 57.4% Verified ≈ $4.60/issue; selection +8.8pp over best member.
- **To Run or Not to Run** — Lin et al. arXiv:2606.26978. V — execution is schedulable cost (1.25pp).
- **Decoding Speculative Decoding** — Yan et al. NAACL 2025; arXiv:2402.01528. V — +111% throughput via draft latency.
- **ProST** — Bijoy et al. IJCNLP-AACL 2025; arXiv:2509.04508. V — progressive sub-task training over a homogeneous 14-14-14 triad beats 32B single at lower FLOPs; heterogeneous triad tested was 14-7-7; Pareto gains require training, not zero-shot cascading (correction per RefuteLit F2).
- **Rethinking Scale (SLM deployment)** — Wang. ACL 2026 Industry. V — <10B MAS coordination tax; completion 99.67→72.01%.
- **When Is Smaller Greener?** — Siraj, Barbati. arXiv:2610.19345. V — crossover task length; per-task (not per-call) economics.
- **SLMs are the Future of Agentic AI** — Belcak et al. (NVIDIA). arXiv:2506.02153. V — position paper.

## Industrial practice

- **Anthropic code-modernization plugin** — claude-plugins-official (Apache 2.0), read at HEAD 2026-10-28. V — 9-phase pipeline; 50–200 rule-extraction agents; evidence-based verify; field runs (Jetty 946 tests; osCommerce PROVEN; CardDemo NOT PROVEN); "state in files not chat"; "two runs find different rules".
- **GitHub Copilot app modernization** — Microsoft Learn (2026-09). V — assess→plan→execute; atomic tasks with build+test gates; Guided mode.
- **AWS Q Transform** — AWS blogs (2023–2026). V — 1,000 Java apps in 2 days by 5 people via OpenRewrite + build/test loop; end-of-support 2026, successor AWS Transform.
- **CodeMenv** — Cheng et al. ACL 2025 Findings. V — migration pass@1 26.5% avg; loc 60–66% vs fix 26%.
- **AgentModernize** — Ahmed. arXiv:2605.17535. V — BER metric; extraction solved (92.3% rule capture), generation is the bottleneck; pipeline benefit inversely correlated with model strength.
- **Articulate but Wrong** — arXiv:2605.21537. V — semantic drift 39.7% on traps; 31.7% self-endorsed.
- **SWE-Bench Pro** — Deng et al. arXiv:2509.16941. V — long-horizon enterprise tasks unsolved.
- **Environment-in-the-Loop** — Li et al. ReCode '26; arXiv:2602.09944. V — environment construction is half the migration problem.
- **Strangler Fig** — Fowler (2004); Microsoft Azure Architecture Center. V — module-at-a-time equivalence.

## Local (repo) evidence

- `docs/references/METHODOLOGY.md`, `METHODOLOGY-DRV.md` (read-only ground truth + derivations).
- `docs/decisions/0001…0017` — ADR ledger (0005 read-only pinning; 0008 LoC tiers; 0009 config schema; 0012 ARIADNE constants; 0016 transport; 0017 language-agnostic pipeline).
- `docs/report/2025-10-25/README.md` — F1–F8 real-campaign failure taxonomy.
- `docs/report/2026-10-28/README.md` + `01–05` — D1–D10 dataset/oracle/granularity analysis, G1–G8 gap mapping.
- `docs/specs/validation-contracts.md` — VAL-* contract system.
- `benchmarks/README.md` — config schema, LoC tiers, acceptance criteria.
- `benchmarks/results/daemon/*.json`, `benchmarks/logs/*-fixverify.log` — campaign receipts.
