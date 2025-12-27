#import sys: inputs

#set par(justify: true)
#set text(lang: "ru")
#set page(
  // paper: "a6", 
  height: auto, 
  header: [К/р. по линейным ур-ям и системам #h(1fr) Вариант #inputs.variant (пример решений).]
)
#set heading(numbering: "1.a.")
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  set text(12pt)
  it
}
#show heading: strong.with(delta: -300);
#show math.eq: math.display
#show math.cases: math.display
#show math.vec: math.display
#show math.mat: math.display

#for (n, task) in inputs.tasks.enumerate() [
  #eval(task.solution, mode: "markup")
]
