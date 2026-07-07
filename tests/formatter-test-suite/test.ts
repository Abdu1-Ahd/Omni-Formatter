// ── CASE 1: Variable declarations with type annotations ────────────────────
const   x :string='hello';
let y: string =  "world" ;
var z:number=42;

// ── CASE 2: Type unions and intersections ──────────────────────────────────
type StringOrNumber = string|number;
type Combined = TypeA & TypeB & {extra: boolean};

// ── CASE 3: Interface definitions — mixed indentation ──────────────────────
interface User{
  id:number;
    name:string;
  email :string;
    createdAt: Date;
}

// ── CASE 4: Generic types — spacing ────────────────────────────────────────
const arr: Array<string> = [];
function identity<T>(arg:T):T { return arg; }
const map = new Map<string,number>();

// ── CASE 5: Enum declarations ──────────────────────────────────────────────
enum Direction{
    Up="UP",
  Down = "DOWN",
    Left = "LEFT",
  Right= "RIGHT",
}

// ── CASE 6: Type assertions and non-null assertions ────────────────────────
const element = document.getElementById("app") as HTMLDivElement;
const value = someValue!;
const forced = (something as unknown) as SpecificType;

// ── CASE 7: Decorators ────────────────────────────────────────────────────
@Component({
  selector: 'app-root',
    template: `<h1>Hello</h1>`,
})
class AppComponent {
  @Input() title: string = '';
    @Output() clicked = new EventEmitter<void>();
}

// ── CASE 8: Long function signature — should break params ──────────────────
function veryLongFunctionCallWithLotsOfArgumentsThatExceedsOneHundredCharacters(arg1: string, arg2: string, arg3: string, arg4: string, arg5: string, arg6: string): void {
  return;
}

// ── CASE 9: Prettier-ignore — must not touch the next statement ────────────
// prettier-ignore
const ignored : number []= [
1,2,3
] ;

// ── CASE 10: Async generic function ────────────────────────────────────────
async function fetchTyped<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, options);
  if (!response.ok) {
    throw new Error(`HTTP error: ${response.status}`);
  }
  return response.json() as Promise<T>;
}

// ── CASE 11: Conditional types ────────────────────────────────────────────
type IsArray<T> = T extends any[] ? true : false;
type Flatten<T> = T extends Array<infer Item> ? Item : T;

// ── CASE 12: Namespace / module augmentation ──────────────────────────────
declare namespace NodeJS {
  interface ProcessEnv {
    NODE_ENV: 'development' | 'production' | 'test';
    PORT?: string;
  }
}

// ── CASE 13: Mixed indent in class methods ────────────────────────────────
class Service {
  private readonly db: Database;
    constructor(db: Database) {
    this.db = db;
  }
    async findById(id: number): Promise<User | null> {
        const result = await this.db.query('SELECT * FROM users WHERE id = $1', [id]);
    return result.rows[0] ?? null;
  }
}

// ── CASE 14: Trailing whitespace on type lines ─────────────────────────────
type Config = {   
  port: number;  
  host: string;   
};
