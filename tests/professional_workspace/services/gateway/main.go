package main

import (
	"net/http"
	"fmt"
	"log"
)

type GatewayConfig struct {
    Port int
      Host string
}

func helloHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Hello from Gateway")
}

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/", helloHandler)
	// very long line exceeding 100 characters in go to test if gofmt wraps it or ignores it as per gofmt standard
	log.Println("Starting gateway server on port 8080 with some extremely long log message that goes on forever")
	http.ListenAndServe(":8080", mux)
}
