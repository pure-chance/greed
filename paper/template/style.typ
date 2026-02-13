#import "packages/theoretic.typ": *
#import "packages/zebraw.typ": *

#let stylize() = contents => {
  // Configure page properties --
  set page(
    columns: 2,
    paper: "a4",
    margin: (x: 1.25cm, y: 1.25cm)
  )
  set columns(gutter: 0.65cm)

  // Configure text properties --
  set text(font: "Libertinus Serif", size: 11pt, weight: "regular")
  show raw: set text(font: "Maple Mono NF")

  // Configure paragraph properties --
  set par(
    spacing: 0.45em,
    justify: true,
    first-line-indent: 1em,
    leading: 0.45em
  )

  // Configure heading properties --
  set heading(numbering: "1.1.")
  show heading: set text(size: 11pt)
  show heading: it => {
    if it.level <= 1 {
      it
    } else {
      // inline heading
      let heading = counter(heading).display(it.numbering) + h(0.2em) + it.body
      block(below: 0pt) + heading + [.]
    }
  }

  show heading.where(level: 1): set text(
    size: 13pt,
    style: "normal",
    weight: "bold"
  )
  show heading.where(level: 2): set text(
    size: 11pt,
    style: "normal",
    weight: "bold"
  )
  show heading.where(level: 3): set text(
    size: 11pt,
    style: "italic",
    weight: "regular"
  )

  // figures & captions --
  show figure: set block(breakable: true)
  show figure.caption: set align(left)
  show figure.where(kind: table): set figure(supplement: [Table])
  show figure.where(kind: raw): set figure(supplement: [Code])
  set figure(supplement: [Fig.])
  set figure.caption(separator: [|])
  show figure.caption: it => [
    #strong[
      #it.supplement
      #context it.counter.display(it.numbering)
      #it.separator
    ]
    #it.body
  ]

  // tables --
  show table.cell.where(y: 0): strong
  set table(
    stroke: (x, y) => if y == 0 {
      (bottom: 0.7pt + black)
    }
  )
  show table: set align(center)


  // math --
  set math.equation(numbering: "(1)")

  // packages --
  show ref: theoretic.show-ref
  show: zebraw.with(indentation: 4)

  contents
}
