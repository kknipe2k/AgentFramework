# Stage 3 — Ship post (v0.1.0 release day)

**When:** the M11 two-path success criterion has been verified end-to-end on a fresh Windows VM by someone other than you. Signed `.msi` is in GitHub Releases. SBOM attached. README updated. Demo video recorded.

**Goal:** announce v0.1.0 is real and shippable. Honest about scope. Repo link only after the post (in a reply or pinned comment), not in the first post — keeps the link from being the headline.

**Format:** longer post (X allows ~25k chars now; use ~600–1200 chars in the main post + a reply with links + an attached demo video).

---

## Draft A — main post

> v0.1.0 of the desktop runtime I've been building is up. Apache 2.0. Windows only. Single-session.
>
> What works: load a framework, run a session, watch the live graph render every agent / tool / skill / verify hook. When the agent needs a capability it doesn't have, the session suspends cleanly — you install the missing piece, resume, the drone replays from the last snapshot.
>
> Build the agent yourself in the workbench: drag-drop palette of Tools / Skills / Agents, generators that write a skill or tool from a sentence (with capability disclosure in plain English at install), sandboxed Tester. Two tiers: Novice reviews every install; Promoted auto-accepts validated artifacts within bounds. Operator tier waits for v1.0.
>
> What doesn't work yet: macOS / Linux (v1.0), full OS-level sandboxing (process boundary in v0.1; seccomp/landlock/sandbox-exec in v1.0), continuous-loop policy, mode router, multi-session, auto-update, Anthropic upstream search UI.
>
> Built openly with Claude Code as the typist; direction, review, and acceptance by me. AI assistance disclosed in every commit. ~6 months elapsed since the spec was first sketched.
>
> Reply for repo + demo + spec.

## Draft A — reply with links

> Repo: github.com/kknipe2k/AgentFramework
>
> Spec: github.com/kknipe2k/AgentFramework/blob/main/agent-runtime-spec.md
>
> 90s demo: [video URL]
>
> What I'd appreciate: bug reports on Windows, second eyes on the threat model in docs/SECURITY.md, anyone who wants to take the Linux port for v1.0 (my son might — would also welcome anyone else).
>
> Issues open. Discussions enabled. Quality > volume on contributions; please read CONTRIBUTING.md before opening a PR.

---

## Draft B — shorter, no preamble

> v0.1.0 Windows Preview is out. Desktop runtime for agentic AI workflows. Live graph + gap detection + capability sandboxing + workbench + generators.
>
> Apache 2.0. Single-session. ~6 months of work.
>
> What works / what doesn't honestly listed in the README. Spec is published in full.
>
> Reply for repo + demo.

---

## Draft C — credit-forward

> v0.1.0 is up. Desktop runtime for agentic AI workflows.
>
> Built on patterns from people who showed the way:
>
> - Boris Cherny's subagent decomposition (orchestrator + analyzer + implementer + verify-app + simplifier)
> - Ralph's autonomous-loop pattern (continuous loop policy lands in v1.0)
> - The shell-based ARIA framework (the reference behavior the runtime's primitives must support)
>
> The differentiator: when an agent needs a capability it doesn't have, the session suspends cleanly and shows you exactly what's missing. No more starting over after an agent flails.
>
> Apache 2.0. Windows first. v1.0 multi-OS in progress.
>
> Spec published in full; honesty about scope built in. Reply for links.

---

## Choose one based on context

- **A** — full disclosure of scope; longer; best for an audience that values honesty over brevity.
- **B** — minimal; works if your account already has audience that knows what to expect.
- **C** — credit-forward; best if the prior-art people you're crediting are likely to see and amplify (organic, not begged for).

## Stylistic rules for the ship post

- **Two posts, not a thread.** Main post + reply with links. Reply-with-links is a pattern X promotes well; threading hides the call to action.
- **Demo video in the main post or the reply, not both.** Pick whichever attaches more cleanly.
- **No "we" if you're solo.** Plural pronouns when there's a team feel performative on solo projects.
- **No corporate-tone calls to action.** "Star the repo" / "Try it out" reads as marketing. The repo link is enough; people who want to try it will.
- **Disclose AI assistance in the main post.** Hidden AI usage discovered later destroys credibility; disclosure is fine and increasingly expected.
- **Limitations in the same breath as features.** Builds trust; protects against "but it doesn't do X" responses.

## After posting — the first 48 hours

1. **Pin the post** (or the reply with links if the reply has the demo).
2. **Reply substantively to substantive questions within 24 hours.** Slow response in week 1 kills a launch.
3. **Don't fish for engagement.** Don't quote-reply your own post; don't "bumping" responses.
4. **Watch issues + discussions.** Tag yourself as the assignee on every issue filed in the first week. Even "won't fix" deserves a response.
5. **Don't promise features in replies.** "Filed as #N, queued" is the right shape, not "yes, doing that next."
6. **First bug report:** respond fast, fix faster. Sets the tone.
7. **Save thoughtful comments / DMs:** these are likely contributors and v0.2 testers.

## Things that signal "doof" — DO NOT include in the ship post

- Logo / brand banner / launch graphic — the demo video is the visual.
- "Excited to share..." — never.
- Listing the tech stack as bragging rights ("Built with Rust 🦀 + Tauri ⚡ + React ⚛️"). The README has the stack; the post is for what the user gets.
- Asking for stars or follows.
- Tagging accounts you don't know personally for amplification.
- Claims you can't back up ("the most secure agent runtime"). Specifics or nothing.
- Drama about why competitors are wrong. Define your project; let theirs be theirs.
- "Built in 6 months while juggling [other thing]" — humblebrag pattern reads as performance.

## Things to be ready for

- "Why Tauri not Electron?" → ADR-0002 link.
- "Why Rust?" → ADR-0002 + ADR-0003.
- "Why Anthropic only?" → "Provider trait is in place; OpenAI/Google/local-Ollama in v2.0+. PR welcome at v1.0."
- "Why no Mac/Linux?" → "Solo maintainer; Windows first; Linux contributor sought for v1.0."
- "Where's [feature]?" → check §0d release scope; reply "scope row N: v[X]" link.
- Trolls, "this is just X with a UI," etc. → don't engage. The post is the answer.

## Pinned-tweet-worthy text

If X allows you to pin one post forever:

> v0.1.0 of a desktop runtime for agentic AI workflows.
>
> Live graph. Gap detection that suspends cleanly instead of letting agents flail. A workbench where novices and experienced users build the same way.
>
> Windows. Apache 2.0. Honest scope.
>
> github.com/kknipe2k/AgentFramework
