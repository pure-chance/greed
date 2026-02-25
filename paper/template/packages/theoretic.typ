#import "@preview/theoretic:0.3.1" as theoretic
#import theoretic.presets.fancy: *

// PRESET = fancy =============================================================

#let rust = oklch(66.47%, 0.1307, 19.23deg)
#let moss = oklch(66.47%, 0.1307, 146.88deg)
#let aqua = oklch(67.33%, 0.1307, 255.18deg)
#let iris = oklch(62.00%, 0.1307, 286.82deg)
#let coal = oklch(38.72%, 0, 286.82deg)

#let claim = claim.with(options: (color: rust))
#let exercise = exercise.with(options: (color: rust))

#let algorithm = algorithm.with(options: (color: moss))
#let axiom = axiom.with(options: (color: moss))
#let definition = definition.with(options: (color: moss))

#let corollary = corollary.with(options: (color: aqua))
#let proposition = proposition.with(options: (color: aqua))
#let lemma = lemma.with(options: (color: aqua))

#let theorem = theorem.with(options: (color: iris))

#let counter-example = counter-example.with(options: (color: coal))
#let example = example.with(options: (color: coal))
#let note = note.with(options: (color: coal))
#let proof = proof.with(options: (color: coal))
#let remark = remark.with(options: (color: coal))

// PRESET = basic =============================================================

// #let spacing-options = (
//   block-args: (
//     above: 1.2em,
//     below: 1.2em,
//   )
// )

// #let algorithm = algorithm.with(options: spacing-options)
// #let axiom = axiom.with(options: spacing-options)
// #let claim = claim.with(options: spacing-options)
// #let corollary = corollary.with(options: spacing-options)
// #let counter-example = counter-example.with(options: spacing-options)
// #let definition = definition.with(options: spacing-options)
// #let example = example.with(options: spacing-options)
// #let exercise = exercise.with(options: spacing-options)
// #let lemma = lemma.with(options: spacing-options)
// #let note = note.with(options: spacing-options)
// #let proof = proof.with(options: spacing-options)
// #let proposition = proposition.with(options: spacing-options)
// #let remark = remark.with(options: spacing-options)
// #let theorem = theorem.with(options: spacing-options)
