const menu=document.querySelector('#menu'), navigation=document.querySelector('#navigation');
function closeNavigation(){menu.setAttribute('aria-expanded','false');navigation.classList.remove('open');}
menu.addEventListener('click',()=>{
  const open=menu.getAttribute('aria-expanded')!=='true';
  menu.setAttribute('aria-expanded',String(open));navigation.classList.toggle('open',open);
  if(open)document.querySelector('#search').focus();
});
document.addEventListener('keydown',event=>{if(event.key==='Escape'){closeNavigation();menu.focus();}});
navigation.addEventListener('click',event=>{if(event.target.closest('a'))closeNavigation();});
document.querySelector('#search').addEventListener('input',event=>{
  for(const link of document.querySelectorAll('.chapter-link'))link.hidden=!link.textContent.toLowerCase().includes(event.target.value.toLowerCase());
});
for(const link of navigation.querySelectorAll('a')){
  if(new URL(link.href).pathname===location.pathname)link.setAttribute('aria-current','page');
}
// Deep links into a collapsed checked claim still reveal the requested text.
function revealAnchor(){
  let target;try{target=document.getElementById(decodeURIComponent(location.hash.slice(1)));}catch{return;}
  if(!target)return;
  for(let node=target.parentElement;node;node=node.parentElement)if(node.tagName==='DETAILS')node.open=true;
  target.scrollIntoView({block:'start'});
}
addEventListener('hashchange',revealAnchor);
if(location.hash)requestAnimationFrame(revealAnchor);
