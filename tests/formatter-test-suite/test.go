package main

import "fmt"
import "os"
import "strings"

// ── CASE 1: Import grouping — gofmt merges single-line imports into a block ─
// (Above are intentionally separated; gofmt should merge them)

// ── CASE 2: Function spacing ──────────────────────────────────────────────
func main ( ) {
    var x = 1
	var y = 2

    fmt.Println(x, y)
}

// ── CASE 3: Mixed indentation (tabs vs spaces) ─────────────────────────────
func mixedIndent() {
    if true {
fmt.Println("bad indent - no indent")
      fmt.Println("bad indent - 6 spaces")
	fmt.Println("correct - tab")
    }
}

// ── CASE 4: Long lines ─────────────────────────────────────────────────────
func longLine() {
    longVariableNameThatForcesALineWrapBecauseItExceedsOneHundredCharactersInTotalLength := 42
    fmt.Println(longVariableNameThatForcesALineWrapBecauseItExceedsOneHundredCharactersInTotalLength, os.Args)
}

// ── CASE 5: Struct definition — spacing ───────────────────────────────────
type User struct {
    ID       int
    Name   string
    Email     string
    CreatedAt string
}

// ── CASE 6: Method on struct ──────────────────────────────────────────────
func (u *User) Greet ( ) string {
    return fmt.Sprintf( "Hello, %s!" , u.Name )
}

// ── CASE 7: Error handling pattern ────────────────────────────────────────
func readFile(path string) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil,err
	}
	return data,nil
}

// ── CASE 8: Goroutines and channels ───────────────────────────────────────
func concurrent() {
    ch := make(chan int,10)
    go func() {
        for i := 0; i < 10; i++ {
            ch <- i
        }
        close(ch)
    }()
    for val := range ch {
        fmt.Println(val)
    }
}

// ── CASE 9: Interface definition ──────────────────────────────────────────
type Repository interface {
    FindByID( id int ) (*User, error)
    Save( user *User ) error
    Delete( id int ) error
}

// ── CASE 10: Trailing whitespace ──────────────────────────────────────────
func withTrailing() {   
    x := 1   
    _ = x   
}

// ── CASE 11: Switch statement ─────────────────────────────────────────────
func describe(i interface{}) string {
    switch v := i.(type) {
    case int:
        return fmt.Sprintf("int: %d", v)
    case string:
        return fmt.Sprintf("string: %s", v)
    default:
        return fmt.Sprintf("unknown: %T", v)
    }
}

// ── CASE 12: Slice operations ─────────────────────────────────────────────
func sliceOps() {
    s := []int{1,2,3,4,5}
    doubled := make([]int,len(s))
    for i,v := range s {
        doubled[i] = v*2
    }
    _ = strings.Join([]string{"a","b","c"}, ",")
    _ = doubled
}
