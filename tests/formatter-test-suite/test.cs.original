using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

// ── CASE 1: Class with properties — mixed spacing ─────────────────────────
public class Person
{
    public int Id{get;set;}
    public string   Name { get; set; }
    public string Email {get;private set;}
    public DateTime CreatedAt { get ; set ; }

    public Person ( int id , string name , string email )
    {
        Id = id;
        Name = name;
        Email = email;
        CreatedAt = DateTime.UtcNow;
    }
}

// ── CASE 2: Interface ─────────────────────────────────────────────────────
public interface IRepository<T> where T : class
{
    Task<T?>FindByIdAsync(int id);
    Task<IEnumerable<T>> GetAllAsync();
    Task SaveAsync(T entity);
    Task DeleteAsync(int id);
}

// ── CASE 3: Async methods with LINQ ──────────────────────────────────────
public class PersonService
{
    private readonly IRepository<Person>  _repo;

    public PersonService(IRepository<Person> repo)=>_repo = repo;

    public async Task<IEnumerable<Person>> GetAdultsAsync()
    {
        var all = await _repo.GetAllAsync();
        return all
            .Where(p => p.CreatedAt >= DateTime.Now.AddYears(-30))
                .OrderBy(p => p.Name)
            .ToList();
    }
}

// ── CASE 4: Extension methods ─────────────────────────────────────────────
public static class StringExtensions
{
    public static bool IsValidEmail(this string email)
    {
        return !string.IsNullOrEmpty(email)&&email.Contains('@');
    }

    public static string ToTitleCase(this string s) =>
        string.IsNullOrEmpty(s) ? s : char.ToUpper(s[0]) + s[1..].ToLower();
}

// ── CASE 5: Switch expression ─────────────────────────────────────────────
public static string Classify(int n) => n switch
{
    0                        => "zero",
    > 0 and < 10             => "single digit",
    >= 10 and < 100          => "two digits",
    _                        => "large",
};

// ── CASE 6: Record types ──────────────────────────────────────────────────
public record Point(double X,double Y)
{
    public double Distance(Point other) =>
        Math.Sqrt(Math.Pow(X - other.X, 2) + Math.Pow(Y - other.Y, 2));
}

// ── CASE 7: Trailing whitespace ───────────────────────────────────────────
public static class Program
{
    public static async Task Main(string[] args)   
    {   
        var person = new Person(1,"Alice","alice@example.com");   
        Console.WriteLine(person.Name.ToTitleCase());   
    }
}
