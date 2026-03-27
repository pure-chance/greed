#import "style.typ": stylize

// imported packages --
#import "packages/theoretic.typ": *
#import "packages/zebraw.typ": *

// template --
#let paper(
  title: [],
  authors: (),
  abstract: [],
  references: none,
  matter,
) = {
  // Set document metadata --
  set document(
    title: title,
    author: authors.join(", "),
    description: abstract,
  )

  show: stylize()

  // frontmatter --
  place(top, float: true, scope: "parent")[
    #let authors = authors.join(", ")
    #set par(first-line-indent: 0pt)
    #text(size: 2em, weight: "bold")[#title] #v(0.2em)
    #text(size: 1.2em, style: "italic")[#authors] #v(0.2em)
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
