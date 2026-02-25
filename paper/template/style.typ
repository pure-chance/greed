#import "packages/theoretic.typ": *
#import "packages/zebraw.typ": *

#let stylize() = contents => {
  // Configure page properties --
  set page(
    columns: 2,
    paper: "a4",
    margin: (x: 12.5mm, y: 12.5mm)
  )
  set columns(gutter: 6mm)

  // Configure text properties --
  set text(font: "Libertinus Sans", size: 10pt, weight: "regular")
  show raw: set text(font: "Maple Mono NF")

  // Configure paragraph properties --
  set par(
    first-line-indent: 1em,
    justify: true,
    spacing: 0.5em,
    leading: 0.5em
  )

  // Configure heading properties --
  set heading(numbering: "1.1.")
  show heading: set text(font: "Libertinus Sans", size: 12pt)
  show heading.where(level: 1): it => {
    set block(above: 1.6em, below: 0.8em)
    it
  }
  show heading: it => {
    if 1 < it.level {
      // inline heading
      set text(size: 10pt)
      let heading = counter(heading).display(it.numbering) + h(0.2em) + it.body
      block(below: 0pt) + heading + [.]
    } else { it }
  }

  show heading.where(level: 1): set text(
    style: "normal",
    weight: "bold"
  )
  show heading.where(level: 2): set text(
    style: "normal",
    weight: "bold"
  )
  show heading.where(level: 3): set text(
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
