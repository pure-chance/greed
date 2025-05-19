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
  set document(title: title, author: authors.join(", "))

  // Configure page properties --
  set page(
    columns: 2,
    paper: "a4",
    margin: (x: 1.5cm, y: 1.5cm)
  )

  // Configure paragraph properties --
  set par(spacing: 0.45em, justify: true, first-line-indent: 1em, leading: 0.45em)

  // Configure heading properties --
  set heading(numbering: "1.A")
  show heading: set text(style: "italic", weight: "regular")

  // figures
  show figure: set block(breakable: true)
  show figure.caption: emph


  // packages --
  show: show-theorion
  show: zebraw.with(indentation: 4)

  // frontmatter --
  place(top, float: true, scope: "parent", clearance: 30pt)[
    #block(width: 100%)[
      #set align(center)
      #text(size: 20pt, weight: "bold")[#title]
      #v(1em)
      #text(size: 14pt)[#authors.join(", ")]
    ]
  ]
  if abstract != none [
    #set text(weight: "bold")
    _Abstract_---#h(weak: true, 0pt)#abstract
  ]

  // matter --
  matter

  // references --
  if references != none {
    references
  }
}
