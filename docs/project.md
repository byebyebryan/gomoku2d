# Project Thesis

Gomoku2D is both a product and a production experiment.

The product goal is straightforward: build a polished, opinionated browser
Gomoku that is worth showing. The production goal is larger: learn how far one
developer can push an old, sentimental project with an AI-centric workflow
while still holding the work to real product standards.

This is not an attempt to build a business around Gomoku. It is an attempt to
learn where agents help, where they fail, what process keeps them useful, and
what kind of product-team surface area they can help cover without lowering the
quality bar.

The first goal still has to be treated honestly. If the game is only a toy
demo, the process lessons are weak. Gomoku2D should be planned and judged
like an alpha/beta product with a coherent release story, clear user value,
strong engineering, and enough personality that it is not just another
five-in-a-row clone.

## The Product / Production Tension

The destination is not the main value, but the destination still has to be
credible.

That means two things are true at once:

- The project is a production experiment. We care about the journey: how
  requirements are shaped, how agents explore code, how designs are reviewed,
  how docs stay aligned, how releases are cut, and how human taste and judgment
  fit into the loop.
- The product cannot be half-hearted. The only useful way to learn this process
  is to treat the product as if it matters in the normal product sense: it
  should have a reason to exist, marketable features, a clean release path, and
  a polished user experience.

This keeps the work from becoming either a throwaway AI demo or a purely
hobby-driven feature pile. The product gives the experiment pressure; the
experiment gives the product discipline.

## Tenets

- **Build like it matters.** Even when the product is not the final objective,
  each release should make sense as a real product milestone.
- **Use product quality as the measurement device.** AI-centric development is
  only interesting here if it can produce something coherent, maintainable, and
  polished.
- **Keep the roadmap product-shaped.** Phases should read like an alpha/beta
  release timeline, not an internal hobby backlog.
- **Prefer character over checkboxes.** Once the foundation is solid, features
  should make Gomoku2D feel distinct instead of merely matching what another
  Gomoku app could do.
- **Keep architecture legible for agents and humans.** Clean boundaries, tests,
  docs, and review notes are not overhead; they are what make repeated agentic
  work possible.
- **Let agents accelerate, not decide.** Agents can explore, implement, test,
  review, and draft. Product judgment, taste, scope control, and final calls
  stay human.
- **Record the process.** Roadmaps, design notes, release docs, asset previews,
  and post-review cleanup are part of the experiment, not just project
  administration.
- **Stay honest about scale.** This is a personal project, not a venture-backed
  service. The stack can be modern and robust without pretending the product
  needs enterprise scope.

## Roadmap Implication

The first phases were allowed to be foundation-heavy:

- `v0.1`: prove Rust + Wasm + browser game viability.
- `v0.2`: turn the prototype into a proper local-first frontend product.
- `v0.3`: add backend foundation for optional identity and cloud continuity.

After that, the roadmap should move toward product identity before filling in
standard online checkboxes. Lab-powered features are the strongest source of
uniqueness: replay analysis, puzzle generation, bot personalities, custom bot
settings, and "save this game" challenges give Gomoku2D a reason to exist that
is stronger than retro styling plus basic online play.

Online play, public sharing, and trusted match records still matter, but they
become more compelling after the game has distinctive moments worth sharing.
