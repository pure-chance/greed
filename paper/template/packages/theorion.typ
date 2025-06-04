#import "@preview/theorion:0.3.3": *
#import cosmos.clouds: *

// pretty colors
#let theorem = theorem.with(fill: rgb("#1e66f5").lighten(80%))
#let proposition = proposition.with(fill: rgb("#7287fd").lighten(80%))
#let definition = definition.with(fill: rgb("#40a02b").lighten(80%))

#let remark = remark.with(fill: rgb("#dd7878"))
#let note = note-box.with(fill: rgb("#1e66f5"))

// important equations
#let equation = theorem-box.with(fill: rgb("#dc8a78").lighten(80%))

// conjecture environment
#let conjecture(body) = theorem-box[body]
