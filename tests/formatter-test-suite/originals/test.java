public class Main {
    // ── CASE 1: Class and field definitions ───────────────────────────────
    private String name;
    private int    age;
    private double   salary;

    public Main ( String name , int age , double salary ) {
        this.name = name;
        this.age = age;
        this.salary = salary;
    }

    // ── CASE 2: Method spacing ────────────────────────────────────────────
    public String getName ( ) { return name; }
    public int getAge(){return age;}
    public double getSalary() { return salary ; }

    // ── CASE 3: Nested if/else — inconsistent indentation ─────────────────
    public String classify() {
        if (salary > 100000) {
            if (age < 30) {
                return "young high earner";
            }
            else {
              return "senior high earner";
            }
        } else if (salary > 50000) {
          return "mid earner";
        } else
        {
            return "entry level";
        }
    }

    // ── CASE 4: Generics ──────────────────────────────────────────────────
    public static <T extends Comparable<T>> T max(T a,T b) {
        return a.compareTo(b) > 0 ? a : b;
    }

    // ── CASE 5: Lambda and streams ────────────────────────────────────────
    public void processNames(java.util.List<String> names) {
        names.stream()
            .filter(n -> n.length() > 3)
            .map(String::toUpperCase)
            .sorted()
                .forEach(System.out::println);
    }

    // ── CASE 6: Long method signature ─────────────────────────────────────
    public static void veryLongMethodNameThatExceedsLineWidth(String parameterOne, String parameterTwo, int parameterThree, double parameterFour) {
        System.out.println(parameterOne + parameterTwo + parameterThree + parameterFour);
    }

    // ── CASE 7: try-with-resources ────────────────────────────────────────
    public String readFile(String path) throws Exception {
        try (java.io.BufferedReader br = new java.io.BufferedReader(new java.io.FileReader(path))) {
            StringBuilder sb = new StringBuilder();
            String line;
            while ((line = br.readLine()) != null) {
                sb.append(line).append('\n');
            }
            return sb.toString();
        }
    }

    // ── CASE 8: Trailing whitespace ────────────────────────────────────────
    public static void main(String[] args) {   
        Main m = new Main("Alice", 30, 75000.0);   
        System.out.println(m.classify());   
    }
}
