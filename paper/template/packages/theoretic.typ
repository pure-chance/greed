#import "@preview/theoretic:0.2.0" as theoretic: theorem, proof, qed

// Add space around math environments.
#let theorem = theorem.with(
  block-args: (
    above: 1em,
    below: 1em,
  )
)

#let proof = proof.with(
  block-args: (
    above: 1em,
    below: 1em,
  )
)

// Add common math environments.
#let proposition = theorem.with(kind: "proposition", supplement: "Proposition")
#let definition = theorem.with(kind: "definition", supplement: "Definition")
#let example = theorem.with(kind: "example", supplement: "Example")

#let remark = theorem.with(supplement: "Remark")
#let note = theorem.with(supplement: "Corollary")

#let equation = theorem.with(supplement: "Equation")
