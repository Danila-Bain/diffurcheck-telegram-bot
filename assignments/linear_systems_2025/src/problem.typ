#import sys: inputs

#set par(justify: true)
#set text(lang: "ru")
#set page(
  margin: (top: 15mm, rest: 5mm),
  height: auto, 
  width: 130mm, 
  header: [К/р. по линейным ур-ям #h(1fr) Вариант #inputs.variant.]
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

#for (n, task) in inputs.tasks.enumerate() [
  = #task.problem

  == #eval(task.equation_homo, mode: "math") #if n == 5 [#eval(task.char_roots, mode: "math")]
  
  == #eval(task.equation, mode: "math")

]
