au BufRead,BufNewFile *.ex,*.exs set filetype=lore
au BufRead,BufNewFile *.eex,*.leex,*.sface set filetype=elore
au BufRead,BufNewFile mix.lock set filetype=lore
au BufRead,BufNewFile * call s:Detectlore()

function! s:DetectLore()
  if (!did_filetype() || &filetype !=# 'lore') && getline(1) =~# '^#!.*\<lore\>'
    set filetype=lore
  endif
endfunction

