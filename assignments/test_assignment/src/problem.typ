#import sys: inputs

#set par(justify: true)
#set text(lang: "ru")
#set page(paper: "a6", height: auto, header: [Мемная к/р. #h(1fr) Вариант #inputs.variant.])
#set heading(numbering: "1.a.")
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  set text(14pt)
  it
}

#for task in inputs.tasks [
  = #task.title

  #task.body
]
