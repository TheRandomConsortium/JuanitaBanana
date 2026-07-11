(function(){
  'use strict';
  if(window.__juanita_form_monitor)return;
  window.__juanita_form_monitor=true;
  if(window.location.protocol==='juanita:')return;

  function capture(form){
    var passEl=form.querySelector('input[type=password]');
    if(!passEl||!passEl.value)return;
    var uEl=form.querySelector(
      'input[type=email],input[autocomplete=email],input[autocomplete=username],'
      +'input[type=text][name*=email i],input[type=text][name*=user i],'
      +'input[type=text][name*=login i],input[type=text][id*=email i],'
      +'input[type=text][id*=user i],input[type=text][id*=login i]'
    );
    if (!uEl) {
      var inputs = Array.from(form.querySelectorAll('input'));
      var pIdx = inputs.indexOf(passEl);
      if (pIdx > 0) {
        for (var i = pIdx - 1; i >= 0; i--) {
          var type = inputs[i].type || 'text';
          if (type === 'text' || type === 'email' || type === 'tel' || type === 'number') {
            uEl = inputs[i];
            break;
          }
        }
      }
    }
    var username=uEl?uEl.value:'';
    if(!username&&!passEl.value)return;
    if(window.webkit&&window.webkit.messageHandlers&&window.webkit.messageHandlers.juanita){
      window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
        type:'credential_capture',
        domain:window.location.hostname,
        username:username,
        password:passEl.value
      }));
    }
  }

  document.addEventListener('submit',function(e){capture(e.target);},true);
})();
