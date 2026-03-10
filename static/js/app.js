// Syntax highlighting is done by highlight.js loaded in view.html
// Optional: confirm before delete is done via onsubmit in list.html
document.addEventListener('DOMContentLoaded', function () {
  if (typeof hljs !== 'undefined') {
    hljs.highlightAll();
  }
  // Mark current nav link (exact path match)
  var path = window.location.pathname.replace(/\/$/, '') || '/';
  document.querySelectorAll('.nav a[href]').forEach(function (a) {
    var href = (a.getAttribute('href') || '/').replace(/\/$/, '') || '/';
    if (href === path) { a.classList.add('current'); }
  });
});
