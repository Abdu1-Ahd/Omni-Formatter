package main
import "fmt"
import "os"

func main ( ) {
    var x = 1
	var y = 2

    // gofmt doesn't support a standard ignore comment, so we won't test it for gofmt compliance
    
    longVariableNameThatForcesALineWrapBecauseItExceedsOneHundredCharactersInTotalLength := 42
    fmt.Println(x, y, longVariableNameThatForcesALineWrapBecauseItExceedsOneHundredCharactersInTotalLength, os.Args)
    
    if true {
fmt.Println("bad indent")
    }
}
