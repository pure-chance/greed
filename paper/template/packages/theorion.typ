#import "@preview/theorion:0.3.3": *
#import cosmos.clouds: *

#let cls = (
  red: oklch(70%, 0.1, 20deg, 40%),
  yellow: oklch(80%, 0.1, 85deg, 40%),
  green: oklch(70%, 0.1, 140deg, 40%),
  teal: oklch(70%, 0.1, 165deg, 40%),
  blue: oklch(70%, 0.1, 260deg, 40%),
  purple: oklch(70%, 0.1, 275deg, 40%),
)

// pretty colors (for environments)
#let theorem = theorem.with(fill: cls.blue, radius: 2pt)
#let proposition = proposition.with(fill: cls.purple, radius: 2pt)
#let definition = definition.with(fill: cls.green, radius: 2pt)
#let proof = proof.with()

// pretty colors (for notes)
#let remark = remark.with(fill: cls.teal.opacify(60%))
#let note = note-box.with(fill: cls.teal.opacify(60%))

// important equations
#let equation = theorem-box.with(fill: cls.yellow, radius: 2pt)
