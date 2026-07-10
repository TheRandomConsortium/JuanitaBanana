(function(){
  var filled=0;
  function fillInput(el, value) {
    if (!el) return;
    el.focus();
    el.value = value;
    var nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
    if (nativeInputValueSetter) {
      nativeInputValueSetter.call(el, value);
    }
    el.dispatchEvent(new Event('focus', { bubbles: true }));
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
    el.blur();
  }

  var uEl = null;
  var uCandidates = document.querySelectorAll(
    'input[type=email],input[autocomplete=email],input[autocomplete=username],'
    +'input[type=text][name*=email i],input[type=text][name*=user i],'
    +'input[type=text][name*=login i],input[type=text][id*=email i],'
    +'input[type=text][id*=user i],input[type=text][id*=login i]'
  );
  if (uCandidates.length > 0) {
    uEl = uCandidates[0];
  } else {
    var pEl = document.querySelector('input[type=password]');
    if (pEl) {
      var inputs = Array.from(document.querySelectorAll('input'));
      var pIdx = inputs.indexOf(pEl);
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
  }
  if(uEl){
    fillInput(uEl, 'USERNAME_PLACEHOLDER');
    filled++;
  }
  var pCandidates = document.querySelectorAll('input[type=password]');
  if(pCandidates.length>0){
    fillInput(pCandidates[0], 'PASSWORD_PLACEHOLDER');
    filled++;
  }
  return filled;
})();
