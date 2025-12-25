#import sys: inputs

#set page(height: auto, margin: (y: 15mm, x: 5mm))
#set page(numbering: "1 / 1")


#set par(justify: true)
#set text(lang: "ru")
#set pagebreak(weak: true)

#[
  #set align(center)
  #set text(15pt)
  
  *#inputs.title*

  #set text(12pt)
  Группа #inputs.group_name
]

Временные рамки:
- Начало: #inputs.available_at
- Время на выполнение: #inputs.duration
- Крайний срок сдачи: #inputs.deadline

#inputs.description

#outline(depth: 1)


#set page(header: context {
  let h = query(selector(heading.where(level: 1)).before(here()))
  if h.len() == 0 {
    h = query(selector(heading.where(level: 1)).after(here())).first()
  } else {
    h = h.last()
  }
  align(center, h.body)
  // let s = query(selector(metadata).after(h.location())).filter(it => it.value.children.first().text  == "h:").first()
  // let short = s.value.children.slice(1).first()
  // align(center, {
  //   h.supplement
  //   " "
  //   if h.numbering != none { numbering(h.numbering, ..counter(heading).at(s.location())) }
  //   [ -- ]
  //   short
  // })
})

#for submission in inputs.submissions [
  #pagebreak()
  = Вариант #submission.variant.number: #submission.student_name

  - Время начала: #submission.started_at
  - Время окончания: #submission.finished_at

  #let problem_n = submission.variant.problems.len();
  #table(
    align: center + horizon, rows: (10mm, 10mm), columns: range(submission.variant.problems.len() + 1).map(i => 1fr),
    ..range(1, problem_n+1).map(i => [#i]),
    [Итог],
  )

  #if submission.variant.problems.len() != 0 [
    #pagebreak()
    #align(center)[== Задачи]
    #for doc in submission.variant.problems [
      #for i in range(1, doc.pages + 1) [
        #pagebreak()
        #image(doc.data, page: i)
      ]
    ]
  ]
  
  #if submission.variant.solutions.len() != 0 [
    #pagebreak()
    #align(center)[== Примеры решений]
    #for doc in submission.variant.solutions [
      #for i in range(1, doc.pages + 1) [
        #pagebreak()
        #image(doc.data, page: i)
      ]
    ]
  ]

  #pagebreak()
  #align(center)[== Присланные решения]
  #if submission.solutions.len() != 0 [
    #for doc in submission.solutions [
      #for i in range(1, doc.pages + 1) [
        #pagebreak()
        #image(doc.data, page: i)
      ]
    ]
  ] else [
    (ничего не прислано)
  ]
]
