#import "@preview/theorion:0.3.3": *
#import cosmos.clouds: *

#let cls = (
  maroon: rgb("#e64553"),
  peach: rgb("#fe640b"),
  yellow: rgb("#df8e1d"),
  green: rgb("#40a02b"),
  blue: rgb("#1e66f5"),
  lavender: rgb("#7287fd"),
)

// pretty colors (for environments)
#let theorem = theorem.with(fill: cls.blue.lighten(80%), radius: 2pt)
#let proposition = proposition.with(fill: cls.blue.lighten(80%), radius: 2pt)
#let definition = definition.with(fill: cls.green.lighten(80%), radius: 2pt)

// pretty colors (for notes)
#let remark = remark.with(fill: cls.maroon.lighten(20%))
#let note = note-box.with(fill: cls.blue.lighten(20%))

// important equations
#let equation = theorem-box.with(fill: cls.maroon.lighten(80%), radius: 2pt)
