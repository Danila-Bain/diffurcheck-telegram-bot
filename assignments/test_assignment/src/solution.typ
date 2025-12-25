#set page(paper: "a6")
#set heading(numbering: "1.a.")
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  it
}
#[
  #set align(center)
  #set text(1.3em)
  *Вариант {{variant}}*
]

= {{task1}}

= {{task2}}
