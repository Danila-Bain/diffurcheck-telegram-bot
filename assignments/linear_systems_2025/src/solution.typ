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
  = #task.problem

  == #eval(task.equation_homo, mode: "math") #if n == 5 [#eval(task.char_roots, mode: "math")]

  Характеристическое уравнение:
  #eval(task.char_equation, mode: "math")
  
  Корни:
  #eval(task.char_roots, mode: "math")

  Решение:
  #eval(task.solution_homo, mode: "math")
  
  == #eval(task.equation, mode: "math")

  Решение:
  #eval(task.solution, mode: "math")

]
