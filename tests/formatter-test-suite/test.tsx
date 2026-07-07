import React, { useState, useCallback, FC, ReactNode } from 'react';

// ── CASE 1: Typed functional component ────────────────────────────────────
interface ButtonProps {
  label:string;
  onClick:(event: React.MouseEvent<HTMLButtonElement>) => void;
  variant?:'primary'|'secondary'|'danger';
  disabled?:boolean;
}

const Button: FC<ButtonProps> = ({ label , onClick , variant = 'primary' , disabled = false }) => {
  return (
    <button
      className={`btn btn-${variant}`}
      onClick={onClick}
      disabled={disabled}
    >
      {label}
    </button>
  );
};

// ── CASE 2: Generic component with constraints ─────────────────────────────
interface ListProps<T extends {id:number;label:string}> {
  items:T[];
  renderItem?:(item:T)=>ReactNode;
}

function TypedList<T extends {id:number;label:string}>({ items , renderItem }:ListProps<T>) {
  return (
    <ul>
      {items.map(item=>(
        <li key={item.id}>
          {renderItem ? renderItem(item) : item.label}
        </li>
      ))}
    </ul>
  );
}

// ── CASE 3: Hook with generics ─────────────────────────────────────────────
function useLocalStorage<T>(key:string, initialValue:T):[T, (value:T)=>void] {
  const [storedValue, setStoredValue] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? (JSON.parse(item) as T) : initialValue;
    } catch {
      return initialValue;
    }
  });

  const setValue = useCallback((value:T)=>{
    setStoredValue(value);
    window.localStorage.setItem(key, JSON.stringify(value));
  },[key]);

  return [storedValue, setValue];
}

// ── CASE 4: Long TSX attribute line ───────────────────────────────────────
function SearchInput() {
  return (
    <input type="search" className="input search-input full-width rounded shadow-md" placeholder="Search across all items in the database" onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleSearch(e.target.value)} />
  );
}

// ── CASE 5: Context with typed provider ───────────────────────────────────
interface ThemeContextValue {
  theme:'light'|'dark';
  toggleTheme:()=>void;
}

const ThemeContext = React.createContext<ThemeContextValue|undefined>(undefined);

function ThemeProvider({children}:{children:ReactNode}) {
  const [theme, setTheme] = useState<'light'|'dark'>('light');
  const toggleTheme = useCallback(()=>{
    setTheme(t=>t === 'light' ? 'dark' : 'light');
  },[]);

  return (
    <ThemeContext.Provider value={{theme,toggleTheme}}>
      {children}
    </ThemeContext.Provider>
  );
}

// ── CASE 6: prettier-ignore ────────────────────────────────────────────────
// prettier-ignore
const rawTable = <table><tr><td>A</td><td>B</td></tr></table>;

export { Button, TypedList, useLocalStorage, SearchInput, ThemeProvider };
