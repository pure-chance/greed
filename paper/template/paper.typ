#import "style.typ": stylize

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

  show: stylize()

  // frontmatter --
  place(top, float: true, scope: "parent")[
    #let authors = authors.join(", ")
    #set par(first-line-indent: 0pt)
    #text(size: 2em, weight: "bold")[#title] #v(0.5em)
    #text(size: 1.2em)[#authors] #v(1em)
  ]

  // abstract --
  if abstract != none [
    #strong[Abstract]---#h(weak: true, 0pt)#abstract
  ]

  // matter --
  matter

  // references --
  if references != none {
    references
  }
}
