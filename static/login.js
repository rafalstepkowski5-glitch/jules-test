async function login(event){
  if (event) {
    event.preventDefault();
  }
  const btn = document.getElementById('loginBtn');
  const originalText = btn.innerHTML;
  btn.disabled = true;
  
  if (!document.getElementById('spinner-style')) {
    const style = document.createElement('style');
    style.id = 'spinner-style';
    style.textContent = '@keyframes spin { to { transform: rotate(360deg); } } .spinner { display: inline-block; width: 12px; height: 12px; border: 2px solid currentColor; border-right-color: transparent; border-radius: 50%; animation: spin 0.75s linear infinite; vertical-align: text-bottom; margin-right: 6px; }';
    document.head.appendChild(style);
  }
  btn.innerHTML = '<span class="spinner"></span>Logging in...';

  const u=document.getElementById('u').value, p=document.getElementById('p').value;
  const csrf = document.cookie.split('; ').find(r=>r.startsWith('csrf_token='))?.split('=')[1];
  try {
    const r=await fetch('/api/auth/login',{method:'POST',headers:{'Content-Type':'application/json','X-CSRF-Token':csrf||''},body:JSON.stringify({username:u,password:p})});
    if(r.ok){ location.href='/'; } else { document.getElementById('msg').innerText='Błąd: '+r.status; }
  } catch (err) {
    document.getElementById('msg').innerText='Błąd: '+err.message;
  } finally {
    btn.disabled = false;
    btn.innerHTML = originalText;
  }
}
document.getElementById('loginForm').addEventListener('submit', login);
