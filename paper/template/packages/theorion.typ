#import "@preview/theorion:0.3.3": *
#import "../colors.typ": *
#import cosmos.clouds: *

// pretty colors (for environments)
#let theorem = theorem.with(fill: hl.blue)
#let proposition = proposition.with(fill: hl.lavender)
#let definition = definition.with(fill: hl.green)

// pretty colors (for notes)
#let remark = remark.with(fill: colors.maroon.lighten(20%))
#let note = note-box.with(fill: colors.blue.lighten(20%))

// important equations
#let equation = theorem-box.with(fill: hl.maroon)
