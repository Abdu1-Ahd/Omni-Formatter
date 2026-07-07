import React, { useState, useEffect } from 'react';

// ── CASE 1: Functional component with props ────────────────────────────────
const Greeting = ({ name , age }) => {
  return (
    <div className="greeting">
      <h1>Hello, {name}!</h1>
      <p>You are {age} years old.</p>
    </div>
  );
};

// ── CASE 2: useState hook — mixed spacing ──────────────────────────────────
function Counter ( ) {
  const [count,setCount] = useState(0);
  const [isLoading , setIsLoading] = useState( false );

  useEffect(()=>{
    document.title = `Count: ${count}`;
  },[count]);

  return (
    <div>
      <button onClick={()=>setCount(c=>c+1)}>Increment</button>
      <span>{count}</span>
    </div>
  );
}

// ── CASE 3: JSX with conditional rendering ─────────────────────────────────
function UserCard ({ user , onDelete }) {
  if (!user) {
    return <div className="empty">No user</div>;
  }

  return (
    <div   className="card"   style={{padding:'16px',margin:'8px'}}>
      <h2>{user.name}</h2>
      {user.email && <p className="email">{user.email}</p>}
      {user.isAdmin ? (
        <span className="badge">Admin</span>
      ) : (
        <span className="badge secondary">User</span>
      )}
      <button onClick={()=>onDelete(user.id)}>Delete</button>
    </div>
  );
}

// ── CASE 4: Long JSX attribute line ───────────────────────────────────────
function Form() {
  return (
    <input type="text" className="input-field primary large rounded shadow" placeholder="Enter your full name here" onChange={(e) => handleChange(e)} onFocus={() => setFocused(true)} />
  );
}

// ── CASE 5: Fragment and list rendering ───────────────────────────────────
function List({items}) {
  return (
    <>
      {items.map((item,index)=>(
        <div key={item.id ?? index} className="list-item">
          <span>{item.label}</span>
        </div>
      ))}
    </>
  );
}

// ── CASE 6: prettier-ignore ────────────────────────────────────────────────
// prettier-ignore
const matrix = <table><tr><td>1</td><td>2</td></tr></table>;

export { Greeting, Counter, UserCard, Form, List };
