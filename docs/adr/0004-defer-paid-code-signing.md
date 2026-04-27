# ADR-0004: Defer Paid Code-Signing for v0.1; Use SHA-256 + Sigstore Provenance Instead

**Status:** Accepted
**Date:** 2026-04-18
**Deciders:** @kknipe2k
**Tags:** distribution, security, oss, scope

## Context

An earlier iteration of `docs/MVP-v0.1.md` and `.github/workflows/release.yml` assumed v0.1 would ship a Windows-EV-code-signed `.msi`. That assumption was carried over from "what a polished commercial desktop product does" without re-evaluating against the project's actual context.

After honest review, the assumption doesn't hold for a v0.1 OSS project:

**Costs of paid EV code-signing for v0.1:**
- $300–600/year recurring vendor fee (SSL.com, Sectigo, DigiCert).
- LLC registration recommended for individual founders to streamline EV verification (otherwise notarized identity docs + sometimes phone interviews; ~$50–200 + 1–2 weeks).
- 2–3 weeks for EV verification on the vendor side.
- USB hardware token shipped to the developer (FIPS 140-2 requirement for new EV certs).
- Local signing only — hardware token can't be loaded into GitHub Actions cleanly without an additional cloud-HSM service (eSigner, ~$100–200/year extra), so signing becomes a manual step on the developer's machine for each release until the cloud-HSM path is set up.
- Procurement risk: if the cert is delayed, ship is delayed (Risk R4 in MVP risk register).

**What a paid EV cert delivers:**
- Windows SmartScreen accepts the binary without warning from first install. Users skip the "Windows protected your PC" dialog entirely.
- For a polished consumer product targeting non-technical users where SmartScreen friction directly bounces installs, this is worth the cost.

**What a paid EV cert does NOT deliver:**
- Cryptographic supply-chain integrity (that's Sigstore / SLSA / SHA-256).
- Trust beyond Windows.
- Verification of *which build process produced the binary* (only verification of *who paid for the cert*).

**What's true for OSS desktop projects in practice:**
- Most successful OSS Tauri/Electron desktop apps ship unsigned at v0.1 and through their first several releases. Reputation accrues over downloads + community trust + transparent build process.
- The OSS community is broadly familiar with the SmartScreen warning and accustomed to clicking through it for unsigned-but-trusted projects.
- Larger OSS desktop projects (VSCode, Cursor, Obsidian) are signed, but those are commercially-backed; community-driven projects often remain unsigned until a sponsor or employer covers the cost.
- Free alternatives exist that handle the supply-chain integrity layer (Sigstore via GitHub Actions OIDC, SHA-256 checksums) — these don't satisfy Windows SmartScreen but they satisfy users who care about provenance and integrity, which is the more security-relevant subset of users at v0.1.

**Project-specific considerations:**
- v0.1 audience is technical (developers experimenting with agentic workflows). They know what SmartScreen is and how to work around it. They also disproportionately care about Sigstore/SLSA-style provenance.
- v0.1 is a Windows Preview, not a polished v1.0 release. Setting expectations correctly — including SmartScreen friction — is part of the v0.1 honesty posture.
- Solo maintainer; no validated audience yet. Paying for things before knowing they're valued is the wrong order.
- Engineering Charter (§12) calls for Sigstore signing + SLSA Level 3 provenance "from v1.0." v0.1 was always going to be a partial implementation; making code-signing follow the same staged path is consistent.

## Decision

For v0.1.0 Windows Preview, we adopt **unsigned Windows installers with SHA-256 checksums + Sigstore provenance attestations** instead of paid EV code-signing.

Specifically:

1. **Build:** `cargo tauri build` produces an unsigned `.msi` for Windows x64. No signing secrets in CI.
2. **Integrity:** GitHub Actions release workflow generates a SHA-256 checksum for every release artifact and publishes it in the GitHub Release notes.
3. **Provenance:** GitHub Actions release workflow uses `actions/attest-build-provenance@v1` (free, OIDC-backed) to produce Sigstore attestations. Verifiable by users with `cosign verify-blob`, which proves "this binary was built by GitHub Actions for this repo from this commit."
4. **User communication:** README install instructions explicitly explain the SmartScreen warning, document the SHA-256 verification step, and document the optional `cosign verify-blob` step for users who want stronger provenance verification.
5. **Build-from-source escape hatch:** README points users who don't want to click through SmartScreen to `cargo tauri build` — they can produce the same artifact locally with no warning.

**When we revisit:**

Paid EV code-signing is reconsidered when **any** of the following becomes true:

- v0.5+ release with measurable adoption (e.g., 1000+ unique downloads of a prior release), at which point friction from SmartScreen meaningfully gates further growth.
- A sponsor, employer, or grant offers to cover the recurring cert cost.
- The project's audience shifts substantively toward non-technical users (Microsoft Store distribution becomes more relevant at that point too — the Store signs apps for free as part of submission).
- A user-reported security concern specifically about unsigned distribution warrants the upgrade.

Until at least one of those triggers, signed builds are explicitly out-of-scope.

## Consequences

### Positive
- v0.1 ships without procurement friction. No LLC registration, no vendor verification wait, no $300–600 outlay before validated demand.
- Sigstore provenance is in place from day one — provides stronger supply-chain integrity than EV code-signing alone (the cert proves identity, not build process).
- SHA-256 checksums in release notes give users a simple integrity check that doesn't require any tooling beyond `Get-FileHash` (Windows) or `sha256sum` (macOS/Linux).
- README posture matches the project's overall honesty stance — explicit about limitations, doesn't pretend SmartScreen friction doesn't exist.
- Users who care about provenance can verify it; users who don't can click through SmartScreen; users who don't trust either can build from source.
- Consistent with §12 Engineering Charter staged rollout (Sigstore + SLSA from v1.0; v0.1 partial).
- Risk register R4 downgraded from "high probability × medium impact, 2-3 weeks of procurement risk" to "high probability × low impact, documented friction."

### Negative
- Every Windows user sees a SmartScreen warning on first install. For non-technical testers this can be alarming and may bounce some installs.
- Project looks less "professional" to first-impression viewers who aren't aware of the OSS context. Mitigated by README copy that explains why.
- More README real estate spent on install instructions than would be needed with a signed build. ~30 lines instead of ~5.

### Neutral / future implications
- Microsoft Store free signing path becomes available later (Tauri supports MSIX packaging). Worth pursuing once the project has audience; not a v0.1 priority.
- If a sponsor offers EV cert coverage at any point, switching is straightforward — the release workflow's signing step is commented out, not removed.
- Sigstore attestation infrastructure is reusable for v1.0 macOS / Linux distributions too (notarytool / GPG signing complement, not replace).

## Alternatives Considered

### Alternative A: Pay for EV code-signing at v0.1
What was originally planned. ~$300–600/year + LLC + 2–3 week procurement.

**Rejected because:** premature optimization for adoption that hasn't been validated. Better to ship, see who shows up, and then invest in distribution polish for the audience that exists rather than guessing.

### Alternative B: Pay for cheaper OV (Organization Validation) code-signing instead of EV
~$200–300/year. No hardware token requirement.

**Rejected because:** OV-signed binaries still trigger SmartScreen warnings until they accumulate ~3000 unique downloads of "reputation," which can take months for a v0.1 OSS project. Pays for the friction without removing it. EV gets you trusted-from-day-1; OV gets you trusted-eventually-maybe.

### Alternative C: Distribute exclusively via Microsoft Store
Microsoft signs Store apps for free.

**Rejected because:** Store submission has its own review friction; adds a distribution channel with its own constraints (MSIX packaging, content review); reaches a different audience (consumers, not developers); not appropriate as the *only* distribution path for a developer-tools v0.1. Worth pursuing as a *supplementary* channel later.

### Alternative D: Self-sign with a free certificate from Let's Encrypt or similar
There's no equivalent to Let's Encrypt for Windows code-signing.

**Rejected because:** technically not possible. Let's Encrypt covers TLS certificates, not Authenticode. Self-signed certificates don't satisfy SmartScreen (Windows doesn't trust unknown roots).

### Alternative E: GPG-sign the release artifact
Traditional OSS approach.

**Rejected because:** no Windows-side integration. GPG signatures are useful but they don't move the needle on SmartScreen and they're less universally tooled than Sigstore for verifying binaries. Sigstore is the modern OSS-native equivalent and we get it for free via GitHub Actions OIDC.

## Related

- Spec section: §0d Release Scope Matrix (distribution row updated)
- Spec section: §12 Engineering Charter (release-rigor staged rollout)
- File: `docs/MVP-v0.1.md` M11 acceptance + risk register R4
- File: `docs/README-v0.1.md` install instructions
- File: `.github/workflows/release.yml` (signing secrets removed, SHA-256 + Sigstore added)
- ADR-0003: Engineering Charter (parent decision; this ADR refines the v0.1 implementation of release rigor)

## Notes

This ADR documents a course-correction made when the user pushed back on the assumption that v0.1 needed a paid code-signing certificate. The pushback was correct; the original plan was importing assumptions from "polished commercial software" into a context where they didn't belong. This is exactly the kind of decision where a durable written rationale matters — six months from now, when someone asks "why is the .msi unsigned?", the answer is here, with the trigger criteria for revisiting clearly stated.

The trigger criteria are intentionally specific (1000+ downloads, sponsor offer, audience shift, reported concern) so that "we should sign this now" is a discussion gated on observable conditions, not on someone's general feeling that signed is more professional.
