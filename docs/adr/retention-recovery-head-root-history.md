# Retention recovery head root history

Status: Accepted

An uncommitted complete head must name a staged root satisfying the same root-history predicate used for root-only and manifest-only recovery. Matching canonical head and manifest records, authenticated selected predecessor bytes, and identical pool inodes do not establish that the candidate generation is the exact successor.

Head planning applies `root_succeeds` before publication effects and returns the existing typed `RootNotSuccessor` refusal on disagreement. The predicate requires generation one without a predecessor for initial or newly inserted namespaces, and the checked successor generation with the selected predecessor digest for existing namespaces.

Already committed recovery retains its cleanup exemption because its selected root is the candidate itself rather than the candidate's predecessor. Requiring that root to succeed itself would reject legitimate interrupted cleanup.

Filesystem regressions construct canonical, mutually consistent root, manifest, and head stages at the pre-head publication boundary with skipped generations, noninitial first publication, and noninitial namespace insertion. They require exact typed refusal and preservation of every retained byte. A bounded candidate-generation sweep includes the unsigned maximum; the evidence receipt states its domain and remaining blind spots. These experiments demonstrate runtime publication behavior without claiming every generation, operating-system schedule, or power-loss outcome.
