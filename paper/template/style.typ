#import "packages/theorion.typ": *
#import "packages/zebraw.typ": *

#let stylize() = contents => {
  // Configure page properties --
  set page(
    columns: 2,
    paper: "a4",
    margin: (x: 1.25cm, y: 1.25cm)
  )

  set columns(gutter: 1.5em)

  // Configure text properties --
  set text(font: "Linux Libertine", size: 11pt, weight: "regular")

  // Configure paragraph properties --
  set par(
    spacing: 0.45em, justify: true, first-line-indent: 1em, leading: 0.45em
  )

  // Configure heading properties --
  set heading(numbering: "1.1.")
  show heading: set block(above: 1.25em)
  show heading.where(level: 1): set text(size: 13pt)
  show heading: it => {
    if it.level >= 2 {
      set text(size: 11pt, style: "italic", weight: "regular")
      let heading = counter(heading).display(it.numbering) + h(0.6em) + it.body
      block(below: 0pt) + heading + [.]
    } else {
      it
    }
  }

  // Configure figures & captions --
  show figure: set block(breakable: true)
  show figure: set figure(supplement: "Fig.")
  show figure.caption: set align(left)
  set figure.caption(separator: [|])
  show figure.caption: it => [
    #strong[
      #it.supplement
      #context it.counter.display(it.numbering)
      #it.separator
    ]
    #it.body
  ]

  // math
  set math.equation(numbering: "(1)")

  // packages --
  show: show-theorion
  show: zebraw.with(indentation: 4)

  contents
}
