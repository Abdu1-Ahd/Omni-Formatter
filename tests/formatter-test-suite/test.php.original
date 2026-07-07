<?php
declare(strict_types=1);

// ── CASE 1: Class — mixed indentation and spacing ─────────────────────────
class User
{
    private int $id;
    private string $name;
    private string  $email;

    public function __construct( int $id , string $name , string $email )
    {
        $this->id    = $id;
        $this->name  = $name;
        $this->email = $email;
    }

    public function getId():int{return $this->id;}
    public function getName() : string { return $this->name ; }

    public function isValid() : bool
    {
        return !empty($this->name) && filter_var($this->email, FILTER_VALIDATE_EMAIL) !== false;
    }
}

// ── CASE 2: Interface and traits ──────────────────────────────────────────
interface Serializable
{
    public function serialize(): string;
    public function unserialize(string $data): void;
}

trait Loggable
{
    public function log(string $message): void
    {
        error_log('[' . get_class($this) . '] ' . $message);
    }
}

// ── CASE 3: Arrow functions and match ─────────────────────────────────────
$double = fn($x) => $x * 2;
$filtered = array_filter([1,2,3,4,5], fn($n) => $n % 2 === 0);

$status = 'active';
$label = match ($status) {
    'active'   => 'Active User',
    'inactive' => 'Inactive User',
    'banned'   => 'Banned User',
    default    => 'Unknown',
};

// ── CASE 4: Named arguments and nullsafe operator ─────────────────────────
function createUser(string $name,string $email,int $age=0,bool $isAdmin=false): User {
    return new User(rand(1,1000), $name, $email);
}

$user = createUser(name: 'Alice', email: 'alice@example.com', age: 30);
$name = $user?->getName() ?? 'Unknown';

// ── CASE 5: Array operations ──────────────────────────────────────────────
$numbers = [1, 2, 3, 4, 5];
$squared = array_map(fn($n) => $n ** 2, $numbers);
$sum     = array_reduce($numbers, fn($carry,$item) => $carry + $item, 0);
$assoc   = ['name' => 'Alice',  'age' => 30,  'email' => 'alice@example.com'];

// ── CASE 6: Long function call ────────────────────────────────────────────
function veryLongFunctionNameThatExceedsLineWidth(string $paramOne, string $paramTwo, int $paramThree, float $paramFour = 0.0): string {
    return sprintf('%s %s %d %f', $paramOne, $paramTwo, $paramThree, $paramFour);
}

// ── CASE 7: Trailing whitespace ───────────────────────────────────────────
function trailingExample(): void {   
    $x = 1;   
    $y = 2;  
    echo $x + $y;
}
