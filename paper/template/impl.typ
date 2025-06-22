// imports --
#import "packages/theorion.typ": *
#import "packages/zebraw.typ": *

// template --
#let paper(
  title: [],
  authors: (),
  abstract: [],
  references: none,
  matter
) = {
  // Set document metadata --
  set document(
    title: title,
    author: authors.join(", "),
    date: datetime(year: 2025, month: 05, day: 30),
    description: abstract,
  )

  // Configure page properties --
  set page(
    columns: 2,
    paper: "a4",
    margin: (x: 1.5cm, y: 1.5cm)
  )

  // Configure text properties --
  set text(size: 10pt, weight: "regular")

  // Configure paragraph properties --
  set par(spacing: 0.45em, justify: true, first-line-indent: 1em, leading: 0.45em)

  // Configure heading properties --
  set heading(numbering: "I.A.")
  show heading: set text(style: "italic", weight: "regular")
  show heading: set block(above: 2em)

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

  // frontmatter --
  place(top, float: true, scope: "parent", clearance: 4em)[
    #block(width: 100%)[
      #set align(center)
      #text(size: 2em, weight: "bold")[#title]
      #v(1em)
      #authors.join(", ")
    ]
  ]

  // abstract --
  if abstract != none [
    #[
      #set align(center)
      #heading(level: 1, numbering: none)[---Abstract---]
    ]
    #abstract
  ]

  // matter --
  matter

  // references --
  if references != none {
    references
  }
}
