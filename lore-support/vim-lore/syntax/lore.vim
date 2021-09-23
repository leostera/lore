" Vim syntax file
" Language: Lore
" Maintainer: Leandro Ostera <leandro@abstractmachines.dev>

if exists("b:current_syntax")
  finish
endif

syn keyword loreLangKeywords as attr in kind prefix rel using

syn match loreLangPrefix '@[a-zA-Z]+[/]?' contained display
syn match loreLangUri '[a-z0-9][a-z0-9-]*:[a-zA-Z0-9()+,-.:=@;$_!*%/?#]+' contained display

syn keyword loreLangTodo TODO FIXME NOTE
syn match loreLangComment "#.*$" contains=loreLangTodo

syn region loreLangString start='"' end='"' contained
syn region loreLangBlock start="{" end="}" fold transparent
