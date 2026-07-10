(function(){
  'use strict';
  if(window.__juanita_form_interact)return;
  window.__juanita_form_interact=true;

  function onFocus(e){
    var el = e.target;
    if(el && el.tagName === 'INPUT') {
      var type = el.type || 'text';
      var isPassword = type === 'password';
      var isUsername = type === 'email' || el.autocomplete === 'username' || el.autocomplete === 'email' || el.name === 'username' || el.name === 'email';
      
      var hasPassword = document.querySelector('input[type=password]') !== null;

      if ((isPassword || isUsername) && hasPassword) {
        if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
          window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
            type: 'form_interact',
            domain: window.location.hostname
          }));
        }
      }
    }
  }

  document.addEventListener('focus', onFocus, true);
  document.addEventListener('click', onFocus, true);
})();
